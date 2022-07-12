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

use cffc_base::api::bm_api::CarNotifyParams;
use cffc_base::api::lane;
use cffc_base::model::img_file;
use cffc_base::util::delay_queue::DelayQueueChan;
use cffc_base::util::utils;

use crate::app_ctx::AppCtx;
use crate::dao::model::{CfCartrack, CfDfsource};
use crate::error::{AppError, AppResult};
use crate::queue_item::{CtQI, NotifyCarQueueItem};
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

    notify: CarNotifyParams,
}

pub enum TrackEvent {
    New,
    APPEND(Box<Track>),
    DELAY(String),
}

pub struct CarHandler {
    ctx: Arc<AppCtx>,
    out: Arc<Queue<CtQI>>,
}

pub struct CarNotifyProcSvc {
    ctx: Arc<AppCtx>,
    queue: Arc<Queue<NotifyCarQueueItem>>,

    spool: SerialPool,
    ready_tx: TkSender<(String, Duration)>,
    ready_rx: TkReceiver<Result<Expired<String>, TkTimeError>>,

    clean_tx: TkSender<(String, Duration)>,
    clean_rx: TkReceiver<Result<Expired<String>, TkTimeError>>,

    track_map: DashMap<String, Arc<SpHolder>>,
}

// ------------------- impls -------------------
impl CarNotifyProcSvc {
    pub fn new(ctx: Arc<AppCtx>, queue: Arc<Queue<NotifyCarQueueItem>>, out: Arc<Queue<CtQI>>) -> Self {
        let handler = CarHandler {
            ctx: ctx.clone(),
            out,
        };

        let spool = SerialPool::new(handler);

        let dq_ready = DelayQueue::new();
        let (ready_tx, ready_rx) = dq_ready.channel();

        let dq_clean = DelayQueue::new();
        let (clean_tx, clean_rx) = dq_clean.channel();

        let track_map = DashMap::new();

        CarNotifyProcSvc {
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

    fn check_recv_mode(&self, _item: &NotifyCarQueueItem) -> bool {
        self.ctx.cfg.notify_proc.cartrack.recv_mode.fast
    }

    async fn process_item(&mut self, item: NotifyCarQueueItem) {
        debug!("CarNotifyProcSvc, process_item:{:?}", item.uuid);

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
            debug!("ct already in spmap, {}", uuid);
        } else {
            // 新的track, 不在spmap中, dispath it
            debug!("ct create to spmap, {}", uuid);
            let holder = Arc::new(SpHolder::new(track));
            self.track_map.insert(uuid.clone(), holder.clone());
            self.spool.dispatch(holder.clone(), TrackEvent::New).await;

            // 放入清除/延时队列中
            let clean_timeout = self.ctx.cfg.notify_proc.cartrack.clear_delay;
            let _ = self.clean_tx.send((uuid.clone(), Duration::from_millis(clean_timeout))).await;
            debug!("ct put to clean_queue, {}", uuid);

            // 如果not ready, 加入 timeout队列
            if !ready {
                let ready_timeout = self.ctx.cfg.notify_proc.cartrack.ready_delay;
                let _ = self.ready_tx.send((uuid.clone(), Duration::from_millis(ready_timeout))).await;
                debug!("ct put to timeout_queue, {}", uuid);
            }
        }
    }

    async fn process_ready_timeout(&self, uuid: String) {
        debug!("ct process ready_timeout: {}", uuid);
        if let Some(holder) = self.track_map.get(&uuid) {
            // 已经在map中
            let holder = holder.value().clone();
            self.spool.dispatch(holder, TrackEvent::DELAY(uuid)).await;
        } else {
            error!("error, ct uuid not in map, {}", uuid);
        }
    }

    async fn process_clean_timeout(&self, uuid: String) {
        debug!("ct process clean_timeout: {}", uuid);
        let _ = self.track_map.remove(&uuid);
        debug!("ct remove uuid: {}", uuid);
    }
}

// ------------------- impl Service -------------------
impl Service for CarNotifyProcSvc {
    fn run(self, rx: Receiver<i64>) -> TkJoinHandle<()> {
        let mut svc = self;
        let mut exit_rx = rx;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    quit = exit_rx.next() => {
                        if let Some(100) = quit {
                            info!("CarNotifyProcSvc recv exit");
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
            info!("CarNotifyProcSvc exit.");
        })
    }
}

// ------------------- impl Handler -------------------
impl CarHandler {
    /// 保存车辆小图/背景图/车牌图到文件中
    /// 新建一条cartrack记录，保存到数据库中
    /// 更新 wp
    /// 记录各种操作耗时
    async fn save_track(&self, track: &mut Track, source_po: &Option<CfDfsource>) -> AppResult<()> {
        let df_imgs = self.ctx.cfg.df_imgs.as_str();
        let track_id = track.uuid.as_str();

        let mut img_ids = String::new();
        let mut img_num = 0;
        let now = Local::now();

        // 准备目录
        let dir = img_file::get_cartrack_imgdir(df_imgs, track_id);
        let _ = utils::prepare_dir(dir).await?;

        // 保存车辆图
        for vehicle in track.notify.vehicles.iter() {
            img_num += 1;
            img_ids.push_str(format!("{}:1.0,", img_num).as_str());

            let fn_small = img_file::get_cartrack_full_imgpath(df_imgs, track_id, img_num);
            // debug!("save vehicle img file: {:?}", fn_small);
            utils::write_jpg_file(fn_small, vehicle.img_buf.bytes()).await?;
        }

        // 保存背景图
        let fn_bg = img_file::get_cartrack_full_bgpath(df_imgs, track_id);
        // debug!("save vehicle bg img file: {:?}", fn_bg);
        utils::write_jpg_file(fn_bg, track.notify.background.image_buf.bytes()).await?;


        // 保存车牌图，（如果有车牌）
        if track.notify.has_plate_info() {
            if let Some(ref plate) = track.notify.plate_info {
                let fn_plate = img_file::get_caretrack_full_platepath(df_imgs, track_id);

                // debug!("save vehicle plate file: {:?}", fn_plate);
                utils::write_jpg_file(fn_plate, plate.img_buf.bytes()).await?;
            }
        }

        // 保存车牌二值图，（如果有车牌二值图）
        if track.notify.has_plate_binary() {
            if let Some(ref plate) = track.notify.plate_info {
                let fn_plate = img_file::get_caretrack_full_platebinary_path(df_imgs, track_id);

                // debug!("save vehicle plate file: {:?}", fn_plate);
                utils::write_jpg_file(fn_plate, plate.binary_buf.bytes()).await?;
            }
        }


        // 保存数据库
        let mut plate_judeged = 0;
        let mut vehicle_judeged = 0;

        if track.notify.has_plate_info() {
            plate_judeged = 1;
        }
        if track.notify.has_props_info() {
            vehicle_judeged = 1;
        }

        let (plate_content, plate_type) = track.notify.get_plate_tuple();
        let (move_direct, car_direct, car_color, car_brand,
            car_top_series, car_series, car_top_type, car_mid_type) = track.notify.get_props_tuple();

        if !img_ids.is_empty() {
            // 去掉最后的 ","
            img_ids.truncate(img_ids.len() - 1);
        }

        let po = CfCartrack {
            id: 0,
            sid: track_id.to_string(),
            src_sid: track.notify.source.clone(),
            img_ids,
            alarmed: 0,
            most_coi: None,
            plate_judged: plate_judeged,
            vehicle_judged: vehicle_judeged,
            move_direct,
            car_direct,
            plate_content,
            plate_confidence: track.notify.get_plate_confidence(),
            plate_type,
            car_color,
            car_brand,
            car_top_series,
            car_series,
            car_top_type,
            car_mid_type,
            tag: None,
            flag: 0,
            obj_id: None,
            submit_id: None,
            submit_time: None,
            is_realtime: 0,
            capture_time: track.ts,
            capture_ts: 0,
            capture_pts: 0,
            lane_num: self.get_lane(&track.notify, source_po) as i32,
            gmt_create: now,
            gmt_modified: now,
        };

        let ctx = self.ctx.clone();
        let saved = tokio::task::spawn_blocking(move || {
            ctx.dao.save_cartrack(&po)
        }).await?;

        let new_id = saved?;
        debug!("saved new cartrack, id:{}, sid:{}", new_id, track_id);

        // 更新 wp
        track.wp = track.notify.vehicles.len();

        Ok(())
    }

    /// 保存车辆小图/背景图/车牌图到文件中
    /// 更新数据库中cartrack记录
    /// 更新 wp
    /// 记录各种操作耗时
    async fn update_track(&self, track: &mut Track, source_po: &Option<CfDfsource>) -> AppResult<()> {
        let df_imgs = self.ctx.cfg.df_imgs.as_str();
        let track_id = track.uuid.as_str();

        let mut img_ids = String::new();
        let mut img_num = 0;
        let now = Local::now();
        let wp_old = track.wp;

        // 准备目录
        let dir = img_file::get_cartrack_imgdir(df_imgs, track_id);
        let _ = utils::prepare_dir(dir).await?;

        // 保存车辆图, (wp指针之前的图片已经保存过)
        for vehicle in track.notify.vehicles.iter() {
            img_num += 1;
            img_ids.push_str(format!("{}:1.0,", img_num).as_str());

            if img_num <= wp_old {
                // 之前图片已经保存过
                continue;
            }

            let fn_small = img_file::get_cartrack_full_imgpath(df_imgs, track_id, img_num as i64);
            // debug!("save vehicle img file: {:?}", fn_small);
            utils::write_jpg_file(fn_small, vehicle.img_buf.bytes()).await?;
        }

        // 保存背景图, 覆盖
        let fn_bg = img_file::get_cartrack_full_bgpath(df_imgs, track_id);
        // debug!("save bg img file: {:?}", fn_bg);
        utils::write_jpg_file(fn_bg, track.notify.background.image_buf.bytes()).await?;


        // 保存车牌图，（如果有车牌）
        if track.notify.has_plate_info() {
            if let Some(ref plate) = track.notify.plate_info {
                let fn_plate = img_file::get_caretrack_full_platepath(df_imgs, track_id);

                // debug!("save vehicle plate file: {:?}", fn_plate);
                utils::write_jpg_file(fn_plate, plate.img_buf.bytes()).await?;
            }
        }

        // 保存车牌二值图，（如果有车牌二值图）
        if track.notify.has_plate_binary() {
            if let Some(ref plate) = track.notify.plate_info {
                let fn_plate = img_file::get_caretrack_full_platebinary_path(df_imgs, track_id);

                // debug!("save vehicle plate file: {:?}", fn_plate);
                utils::write_jpg_file(fn_plate, plate.binary_buf.bytes()).await?;
            }
        }


        // 更新数据库
        let mut plate_judeged = 0;
        let mut vehicle_judeged = 0;

        if track.notify.has_plate_info() {
            plate_judeged = 1;
        }
        if track.notify.has_props_info() {
            vehicle_judeged = 1;
        }

        let (plate_content, plate_type) = track.notify.get_plate_tuple();
        let (move_direct, car_direct, car_color, car_brand,
            car_top_series, car_series, car_top_type, car_mid_type) = track.notify.get_props_tuple();


        if !img_ids.is_empty() {
            // 去掉最后的 ","
            img_ids.truncate(img_ids.len() - 1);
        }

        let po = CfCartrack {
            id: 0,
            sid: track_id.to_string(),
            src_sid: track.notify.source.clone(),
            img_ids,
            alarmed: 0,
            most_coi: None,
            plate_judged: plate_judeged,
            vehicle_judged: vehicle_judeged,
            move_direct,
            car_direct,
            plate_content,
            plate_confidence: track.notify.get_plate_confidence(),
            plate_type,
            car_color,
            car_brand,
            car_top_series,
            car_series,
            car_top_type,
            car_mid_type,
            tag: None,
            flag: 0,
            obj_id: None,
            submit_id: None,
            submit_time: None,
            is_realtime: 0,
            capture_time: track.ts,
            capture_ts: 0,
            capture_pts: 0,
            lane_num: self.get_lane(&track.notify, source_po) as i32,
            gmt_create: now,
            gmt_modified: now,
        };

        let ctx = self.ctx.clone();
        let affect = tokio::task::spawn_blocking(move || {
            ctx.dao.upate_cartrack_for_append(&po)
        }).await?;

        let affect = affect?;
        let updated = affect == 1;
        debug!("update cartrack, {}, sid:{}", updated, track_id);
        if !updated {
            return Err(AppError::new(&format!("update CfCartrack, affect rows:{}, {}", affect, track_id)));
        }

        // 更新 wp
        track.wp = track.notify.vehicles.len();

        Ok(())
    }

    /// 查询 source
    /// 生成 CtQI 放入后续队列中
    async fn put_to_next(&self, track: &Track, source_po: &Option<CfDfsource>) -> AppResult<()> {
        let qi = CtQI::from_notify(&self.ctx.cfg.dfimg_url, track.ts, &track.notify, source_po);
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

        // debug!("ct sp process: {}, events len: {}, ready_old:{}", data.uuid, events.len(), ready_old);
        for event in events.into_iter() {
            match event {
                TrackEvent::New => {
                    newed = true;
                }
                TrackEvent::APPEND(mut track) => {
                    appended = true;
                    // 替换背景图，增加图片，车牌图，属性
                    data.notify.background = track.notify.background;
                    data.notify.vehicles.append(&mut track.notify.vehicles);
                    if track.notify.plate_info.is_some() {
                        data.notify.plate_info = track.notify.plate_info;
                    }
                    if track.notify.props.is_some() {
                        data.notify.props = track.notify.props;
                    }
                }
                TrackEvent::DELAY(_) => {
                    delayed = true;
                }
            }
        }

        // 查询摄像头
        let ctx = self.ctx.clone();
        let source_id = data.notify.source.clone();
        let source_po = tokio::task::spawn_blocking(move || {
            ctx.dao.load_source_by_sid(&source_id)
        }).await;
        if let Err(e) = source_po {
            error!("error, CarHandler load_source_by_sid, {}, {:?}", data.notify.source, e);
            return;
        }
        let source_po = source_po.unwrap();
        if let Err(e) = source_po {
            error!("error, CarHandler load_source_by_sid, {}, {:?}", data.notify.source, e);
            return;
        }
        let source_po = source_po.unwrap();


        // 新建
        if newed {
            if let Err(e) = self.save_track(&mut data, &source_po).await {
                error!("error, CarHandler save_track, {}, {:?}", data.uuid, e);
                data.invalid = true; //保存失败
                return;
            } else {
                debug!("CarHandler save_track ok, {}", data.uuid);
            }
        }

        // 更新
        if !newed && appended {
            if let Err(e) = self.update_track(&mut data, &source_po).await {
                error!("error, CarHandler update_track, {}, {:?}", data.uuid, e);
                return;
            } else {
                debug!("CarHandler update_track ok, {}", data.uuid);
            }
        }

        if appended || delayed {
            data.ready_flag = true;
        }
        let ready_new = data.ready_flag;

        if (newed && ready_old) || (!ready_old && ready_new) {
            if data.invalid {
                error!("error, cartrack:{} is ready, but invalid, skip it", data.uuid);
            } else {
                // 交给后续队列处理
                if let Err(e) = self.put_to_next(&data, &source_po).await {
                    error!("error, CarHandler put_to_next, {}, {:?}", data.uuid, e);
                } else {
                    debug!("CarHandler put_to_next ok, {}", data.uuid);
                }
            }
        }

        // 清除图片
        data.notify.clear_blob();
    }

    fn get_lane(&self, notify: &CarNotifyParams, source_po: &Option<CfDfsource>) -> usize {
        let po = source_po.as_ref();
        if po.is_none() {
            return 0;
        }

        let po = po.unwrap();

        /*'摄像头拍摄目标的方向 0:未知 1:正面 2:侧面 3:后面'*/
        let same_direct = match po.direction {
            0 | 2 => {
                return 0;
            }
            1 => false,
            3 => true,
            _ => {
                return 0;
            }
        };

        let mut lane_desc = None;
        if let Some(v) = source_po {
            if let Some(ref desc) = v.lane_desc {
                lane_desc = Some(desc.clone());
            }
        }
        if lane_desc.is_none() {
            return 0;
        }
        let lane_desc = lane_desc.unwrap();

        // 根据bg图，计算出在视频中的坐标，因为车道线是按照视频（大小）的坐标来标注的


        let scale_x: f64 = notify.background.video_width as f64 / notify.background.width as f64;
        let scale_y: f64 = notify.background.video_height as f64 / notify.background.height as f64;

        let x = (notify.background.rect.x + notify.background.rect.w / 2) as f64 * scale_x;
        let y = (notify.background.rect.y + notify.background.rect.h / 2) as f64 * scale_y;


        let lane_num = lane::get_vehicle_lane_fromstr(x as i64, y as i64, &lane_desc, same_direct);
        if let Err(e) = lane_num {
            error!("error, CarHandler, get_vehicle_lane_fromstr:{}, {}", lane_desc, e);
            return 0;
        }

        lane_num.unwrap()
    }
}