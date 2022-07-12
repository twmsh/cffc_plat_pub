use std::sync::Arc;
use std::time::Duration;

use bytes::Buf;
use chrono::prelude::*;
use dashmap::DashMap;
use deadqueue::unlimited::Queue;
use log::{debug, error, info};
use tokio::stream::StreamExt;
use tokio::sync::mpsc::{Receiver as TkReceiver, Sender as TkSender};
use tokio::sync::Mutex;
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle as TkJoinHandle;
use tokio::time::{delay_queue::{DelayQueue, Expired}, Error as TkTimeError};

use cffc_base::api::bm_api::FaceNotifyParams;
use cffc_base::model::img_file;
use cffc_base::util::delay_queue::DelayQueueChan;
use cffc_base::util::utils;

use crate::app_ctx::AppCtx;
use crate::dao::model::CfFacetrack;
use crate::error::{AppError, AppResult};
use crate::queue_item::{FtQI, NotifyFaceQueueItem};
use crate::services::Service;

use super::spool_async::{SerialPool, SpHolder};

// ------------------- structs -------------------
pub struct Track {
    uuid: String,
    ts: DateTime<Local>,
    invalid: bool,
    ready_flag: bool,

    /// 写指针，代表下一个要被保存到磁盘的图片index
    wp: usize,

    notify: FaceNotifyParams,
}

pub enum TrackEvent {
    New,
    APPEND(Box<Track>),
    DELAY(String),
}

pub struct FaceHandler {
    ctx: Arc<AppCtx>,
    out: Arc<Queue<FtQI>>,
}

pub struct FaceNotifyProcSvc {
    ctx: Arc<AppCtx>,
    queue: Arc<Queue<NotifyFaceQueueItem>>,

    spool: SerialPool,
    ready_tx: TkSender<(String, Duration)>,
    ready_rx: TkReceiver<Result<Expired<String>, TkTimeError>>,

    clean_tx: TkSender<(String, Duration)>,
    clean_rx: TkReceiver<Result<Expired<String>, TkTimeError>>,

    track_map: DashMap<String, Arc<SpHolder>>,
}

// ------------------- impls -------------------
impl FaceNotifyProcSvc {
    pub fn new(ctx: Arc<AppCtx>, queue: Arc<Queue<NotifyFaceQueueItem>>, out: Arc<Queue<FtQI>>) -> Self {
        let handler = FaceHandler {
            ctx: ctx.clone(),
            out,
        };

        let spool = SerialPool::new(handler);

        let dq_ready = DelayQueue::new();
        let (ready_tx, ready_rx) = dq_ready.channel();

        let dq_clean = DelayQueue::new();
        let (clean_tx, clean_rx) = dq_clean.channel();

        let track_map = DashMap::new();

        FaceNotifyProcSvc {
            ctx,
            queue,
            spool,
            ready_tx,
            ready_rx,
            clean_tx,
            clean_rx,
            track_map,
        }
    }

    fn check_recv_mode(&self, item: &NotifyFaceQueueItem) -> bool {
        let fast_mode = self.ctx.cfg.notify_proc.facetrack.recv_mode.fast;
        let count = self.ctx.cfg.notify_proc.facetrack.recv_mode.count;
        let quality = self.ctx.cfg.notify_proc.facetrack.recv_mode.quality;

        if fast_mode {
            return true;
        }

        // 图片数量和质量满足要求,含有特征值
        let cc = item.notify.faces.iter().fold(0_usize, |acc, x| {
            if x.quality > quality && x.feature_buf.is_some() {
                acc + 1
            } else {
                acc
            }
        });

        cc >= count
    }

    async fn process_item(&mut self, item: NotifyFaceQueueItem) {
        debug!("FaceNotifyProcSvc, process_item:{:?}", item.uuid);

        let uuid = item.uuid.clone();
        let ready = self.check_recv_mode(&item);
        let track = Track {
            ready_flag: ready,
            uuid: item.uuid,
            ts: item.ts,
            invalid: false,
            wp: 0,
            notify: item.notify,
        };

        if let Some(holder) = self.track_map.get(uuid.as_str()) {
            // 已经在 spmap中, dispath it
            self.spool.dispatch(holder.clone(), TrackEvent::APPEND(Box::new(track))).await;
            debug!("already in spmap, {}", uuid);
        } else {
            // 新的track, 不在spmap中, dispath it
            debug!("create to spmap, {}", uuid);
            let holder = Arc::new(SpHolder::new(track));
            self.track_map.insert(uuid.clone(), holder.clone());
            self.spool.dispatch(holder.clone(), TrackEvent::New).await;

            // 放入清除/延时队列中
            let clean_timeout = self.ctx.cfg.notify_proc.facetrack.clear_delay;
            let _ = self.clean_tx.send((uuid.clone(), Duration::from_millis(clean_timeout))).await;
            debug!("put to clean_queue, {}", uuid);

            // 如果not ready, 加入 timeout队列
            if !ready {
                let ready_timeout = self.ctx.cfg.notify_proc.facetrack.ready_delay;
                let _ = self.ready_tx.send((uuid.clone(), Duration::from_millis(ready_timeout))).await;
                debug!("put to timeout_queue, {}", uuid);
            }
        }
    }

    async fn process_ready_timeout(&self, uuid: String) {
        debug!("process ready_timeout: {}", uuid);
        if let Some(holder) = self.track_map.get(&uuid) {
            // 已经在map中
            let holder = holder.value().clone();
            self.spool.dispatch(holder, TrackEvent::DELAY(uuid)).await;
        } else {
            error!("error, uuid not in map, {}", uuid);
        }
    }

    async fn process_clean_timeout(&self, uuid: String) {
        debug!("process clean_timeout: {}", uuid);
        let _ = self.track_map.remove(&uuid);
        debug!("remove uuid: {}", uuid);
    }
}

// ------------------- impl Service -------------------
impl Service for FaceNotifyProcSvc {
    fn run(self, rx: Receiver<i64>) -> TkJoinHandle<()> {
        let mut svc = self;
        let mut exit_rx = rx;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    quit = exit_rx.next() => {
                        if let Some(100) = quit {
                            info!("FaceNotifyProcSvc recv exit");
                            break;
                        }
                    }
                    item = svc.queue.pop() => {
                        svc.process_item(item).await;
                    }
                    Some(Ok(expired)) = svc.ready_rx.next() => {
                        svc.process_ready_timeout(expired.into_inner()).await;
                    }
                    Some(Ok(expired)) = svc.clean_rx.next() => {
                        svc.process_clean_timeout(expired.into_inner()).await;
                    }
                }
            }
            info!("FaceNotifyProcSvc exit.");
        })
    }
}

// ------------------- impl Handler -------------------
impl FaceHandler {
    /// 保存人脸大图/小图/背景图到文件中
    /// 新建一条facetrack记录，保存到数据库中
    /// 更新 wp
    /// 记录各种操作耗时
    async fn save_track(&self, track: &mut Track) -> AppResult<()> {
        let df_imgs = self.ctx.cfg.df_imgs.as_str();
        let track_id = track.uuid.as_str();

        let mut img_ids = String::new();
        let mut img_num = 0;
        let now = Local::now();

        // 准备目录
        let dir = img_file::get_facetrack_imgdir(df_imgs, track_id);
        let _ = utils::prepare_dir(dir).await?;

        // 保存人脸图
        for face in track.notify.faces.iter() {
            img_num += 1;
            img_ids.push_str(format!("{}:{},", img_num, face.quality).as_str());

            let fn_small = img_file::get_facetrack_small_imgpath(df_imgs, track_id, img_num);
            let fn_large = img_file::get_facetrack_large_imgpath(df_imgs, track_id, img_num);

            // debug!("save small img file: {:?}", fn_small);
            utils::write_jpg_file(fn_small, face.aligned_buf.bytes()).await?;
            // debug!("save large img file: {:?}", fn_large);
            utils::write_jpg_file(fn_large, face.display_buf.bytes()).await?;
        }

        // 保存背景图
        let fn_bg = img_file::get_facetrack_full_bgpath(df_imgs, track_id);
        // debug!("save bg img file: {:?}", fn_bg);
        utils::write_jpg_file(fn_bg, track.notify.background.image_buf.bytes()).await?;


        // 保存数据库
        let mut gender = 0; // 0 不确定; 1 男性; 2 ⼥性
        let mut age = 0;
        let mut glasses = 0; // 0 不戴眼镜; 1 墨镜; 2 普通眼镜
        let mut direction = 0; // 0 未知；1 向上；2 向下
        if let Some(ref v) = track.notify.props {
            gender = v.gender;
            age = v.age;
            glasses = v.glasses;
            direction = v.move_direction;
        }
        if !img_ids.is_empty() {
            // 去掉最后的 ","
            img_ids.truncate(img_ids.len() - 1);
        }

        let po = CfFacetrack {
            id: 0,
            ft_sid: track_id.to_string(),
            src_sid: track.notify.source.clone(),
            img_ids,
            matched: Some(0),
            judged: Some(0),
            alarmed: Some(0),
            most_person: None,
            most_score: None,
            gender: Some(gender as i32),
            age: Some(age as i32),
            glasses: Some(glasses as i32),
            direction: Some(direction as i32),
            plane_score: None,
            mask: None,
            moustache: None,
            hat: None,
            tag: None,
            flag: 0,
            db_flag: None,
            db_sid: None,
            feature_ids: None,
            obj_id: None,
            submit_id: None,
            submit_time: None,
            capture_time: track.ts,
            gmt_create: now,
            gmt_modified: now,
        };

        let ctx = self.ctx.clone();
        let saved = tokio::task::spawn_blocking(move || {
            ctx.dao.save_facetrack(&po)
        }).await?;

        let new_id = saved?;
        debug!("saved new facetrack, id:{}, sid:{}", new_id, track_id);

        // 更新 wp
        track.wp = track.notify.faces.len();

        Ok(())
    }

    /// 保存人脸大图/小图/背景图到文件中
    /// 更新数据库中facetrack记录
    /// 更新 wp
    /// 记录各种操作耗时
    async fn update_track(&self, track: &mut Track) -> AppResult<()> {
        let df_imgs = self.ctx.cfg.df_imgs.as_str();
        let track_id = track.uuid.as_str();

        let mut img_ids = String::new();
        let mut img_num = 0;
        let now = Local::now();
        let wp_old = track.wp;

        // 准备目录
        let dir = img_file::get_facetrack_imgdir(df_imgs, track_id);
        let _ = utils::prepare_dir(dir).await?;

        // 保存人脸图, (wp指针之前的图片已经保存过)
        for face in track.notify.faces.iter() {
            img_num += 1;
            img_ids.push_str(format!("{}:{},", img_num, face.quality).as_str());

            if img_num <= wp_old {
                // 之前图片已经保存过
                continue;
            }

            let fn_small = img_file::get_facetrack_small_imgpath(df_imgs, track_id, img_num as i64);
            let fn_large = img_file::get_facetrack_large_imgpath(df_imgs, track_id, img_num as i64);

            // debug!("save small img file: {:?}", fn_small);
            utils::write_jpg_file(fn_small, face.aligned_buf.bytes()).await?;
            // debug!("save large img file: {:?}", fn_large);
            utils::write_jpg_file(fn_large, face.display_buf.bytes()).await?;
        }

        // 保存背景图, 覆盖
        let fn_bg = img_file::get_facetrack_full_bgpath(df_imgs, track_id);
        // debug!("save bg img file: {:?}", fn_bg);
        utils::write_jpg_file(fn_bg, track.notify.background.image_buf.bytes()).await?;


        // 保存数据库
        let mut gender = 0; // 0 不确定; 1 男性; 2 ⼥性
        let mut age = 0;
        let mut glasses = 0; // 0 不戴眼镜; 1 墨镜; 2 普通眼镜
        let mut direction = 0; // 0 未知；1 向上；2 向下
        if let Some(ref v) = track.notify.props {
            gender = v.gender;
            age = v.age;
            glasses = v.glasses;
            direction = v.move_direction;
        }
        if !img_ids.is_empty() {
            // 去掉最后的 ","
            img_ids.truncate(img_ids.len() - 1);
        }

        let po = CfFacetrack {
            id: 0,
            ft_sid: track_id.to_string(),
            src_sid: track.notify.source.clone(),
            img_ids,
            matched: None,
            judged: None,
            alarmed: None,
            most_person: None,
            most_score: None,
            gender: Some(gender as i32),
            age: Some(age as i32),
            glasses: Some(glasses as i32),
            direction: Some(direction as i32),
            plane_score: None,
            mask: None,
            moustache: None,
            hat: None,
            tag: None,
            flag: 0,
            db_flag: None,
            db_sid: None,
            feature_ids: None,
            obj_id: None,
            submit_id: None,
            submit_time: None,
            capture_time: track.ts,
            gmt_create: now,
            gmt_modified: now,
        };

        let ctx = self.ctx.clone();
        let affect = tokio::task::spawn_blocking(move || {
            ctx.dao.upate_facetrack_for_append(&po)
        }).await?;

        let affect = affect?;
        let updated = affect == 1;
        debug!("update facetrack, {}, sid:{}", updated, track_id);
        if !updated {
            return Err(AppError::new(&format!("update CfFacetrack, affect rows:{}, {}", affect, track_id)));
        }

        // 更新 wp
        track.wp = track.notify.faces.len();

        Ok(())
    }

    /// 查询 source
    /// 生成 FtQI 放入后续队列中
    async fn put_to_next(&self, track: &Track) -> AppResult<()> {
        let ctx = self.ctx.clone();
        let source_id = track.notify.source.clone();
        let source_po = tokio::task::spawn_blocking(move || {
            ctx.dao.load_source_by_sid(&source_id)
        }).await?;

        let source_po = source_po?;
        let qi = FtQI::from_notify(&self.ctx.cfg.dfimg_url, track.ts, &track.notify, &source_po);
        self.out.push(qi);

        Ok(())
    }

    pub async fn process(&self, holder_data: Arc<Mutex<Track>>, events: Vec<TrackEvent>) {
        let mut newed = false;
        let mut appended = false;
        let mut delayed = false;
        let ready_old;

        let mut data = holder_data.lock().await;
        ready_old = data.ready_flag;

        // debug!("sp process: {}, events len: {}, ready_old:{}", data.uuid, events.len(), ready_old);
        for event in events.into_iter() {
            match event {
                TrackEvent::New => {
                    newed = true;
                }
                TrackEvent::APPEND(mut track) => {
                    appended = true;
                    // 替换背景图，增加图片
                    data.notify.background = track.notify.background;
                    data.notify.faces.append(&mut track.notify.faces);
                }
                TrackEvent::DELAY(_) => {
                    delayed = true;
                }
            }
        }


        // 新建
        if newed {
            if let Err(e) = self.save_track(&mut data).await {
                error!("error, FaceHandler save_track, {}, {:?}", data.uuid, e);
                data.invalid = true; //保存失败
                return;
            } else {
                debug!("FaceHandler save_track ok, {}", data.uuid);
            }
        }

        // 更新
        if !newed && appended {
            if let Err(e) = self.update_track(&mut data).await {
                error!("error, FaceHandler update_track, {}, {:?}", data.uuid, e);
                return;
            } else {
                debug!("FaceHandler update_track ok, {}", data.uuid);
            }
        }

        if appended || delayed {
            data.ready_flag = true;
        }
        let ready_new = data.ready_flag;

        if (newed && ready_old) || (!ready_old && ready_new) {
            if data.invalid {
                error!("error, facetrack:{} is ready, but invalid, skip it", data.uuid);
            } else {
                // 交给后续队列处理
                if let Err(e) = self.put_to_next(&data).await {
                    error!("error, FaceHandler put_to_next, {}, {:?}", data.uuid, e);
                } else {
                    debug!("FaceHandler put_to_next ok, {}", data.uuid);
                }
            }
        }

        // 清除图片及feature内存块
        data.notify.clear_blob(ready_new);
    }
}