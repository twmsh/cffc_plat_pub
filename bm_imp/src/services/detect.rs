use std::sync::Arc;

use chrono::prelude::*;
use deadqueue::unlimited::Queue;
use log::{debug, error};
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender as MSender;
use tokio::task::JoinHandle;
use uuid::Uuid;

use cffc_base::api::bm_api::RecognitionApi;
use crate::cfg::{AppCtx, FeaItem, StageEvent, TaskStat};
use crate::dir_filter::FileItem;
use crate::util;

use super::Service;
use std::path::PathBuf;

/// 多线程调用api获取feature
/// 放入到后续队列中
pub struct DetectService {
    pub ctx: Arc<AppCtx>,
    pub files: Arc<Queue<FileItem>>,
    pub stat: TaskStat,
    pub out: Arc<Queue<FeaItem>>,
    pub api: RecognitionApi,

    pub stage_id: usize,
    /// workder id
    pub stage_wid: usize,

    pub stat_sender: MSender<StageEvent>,
}

impl DetectService {
    pub fn new(ctx: Arc<AppCtx>, files: Arc<Queue<FileItem>>,
               url: &str, out: Arc<Queue<FeaItem>>,
               stage_wid: usize) -> Self {
        let api = RecognitionApi::new(url);

        DetectService {
            stat_sender: ctx.stat_tx.clone(),
            ctx,
            files,
            stat: Default::default(),
            api,
            out,
            stage_id: 0,
            stage_wid,
        }
    }

    async fn send_stat(&mut self, succ: usize, fail: usize) {
        let rs = self.stat_sender.send(StageEvent {
            stage_id: self.stage_id,
            worker_id: self.stage_wid,
            succ,
            fail,
        }).await;

        if let Err(e) = rs {
            error!("error, DetectService.send_stat, {:?}", e);
        }
    }


    async fn do_queue(&mut self, item: FileItem) {
        let time_start = Local::now();

        debug!("DetectService, do_queue:{:?}", item);
        self.stat.count += 1;

        let mut path = PathBuf::from(self.ctx.cfg.imp.img_dir.as_str());
        path.push(item.file_name.as_os_str());

        let content = match util::read_file_base64(path).await {
            Ok(v) => v,
            Err(e) => {
                error!("error, read: [{}]{:?}, {}", item.index, item.file_name, e);
                self.send_stat(0, 1).await;
                return;
            }
        };

        // detect
        let time_1 = Local::now();
        let res = match self.api.detect(content, true, false).await {
            Ok(v) => v,
            Err(e) => {
                error!("error, detect: [{}]{:?}, {:?}", item.index, item.file_name, e);
                self.send_stat(0, 1).await;
                return;
            }
        };
        let time_2 = Local::now();

        let succ_dur = time_2.signed_duration_since(time_1).num_milliseconds();

        if res.code != 0 {
            error!("error, detect: [{}]{:?}, return code:{}, msg:{}",
                   item.index, item.file_name, res.code, res.msg);
            self.send_stat(0, 1).await;
            return;
        }
        let faces = match res.faces {
            Some(v) => v,
            None => {
                error!("error, can't find face: [{}]{:?}",
                       item.index, item.file_name);
                self.send_stat(0, 1).await;
                return;
            }
        };

        let face = match faces.get(0) {
            Some(v) => v,
            None => {
                error!("error, can't find face: [{}]{:?}",
                       item.index, item.file_name);
                self.send_stat(0, 1).await;
                return;
            }
        };
        let feature = match face.feature {
            Some(ref v) => v,
            None => {
                error!("error, no feature: [{}]{:?}",
                       item.index, item.file_name);
                self.send_stat(0, 1).await;
                return;
            }
        };

        // 生成uuid, 保存align
        let person_id = Uuid::new_v4().to_string();
        let _ = match self.save_align_img(person_id.as_str(), face.aligned.as_str()).await {
            Ok(v) => v,
            Err(e) => {
                error!("error, save aligin img fail, {:?}", e);
                self.send_stat(0, 1).await;
                return;
            }
        };

        // 放入后续队列中
        self.out.push(FeaItem {
            index: item.index,
            file_name: item.file_name.clone(),
            person_id,
            fea: feature.clone(),
            score: face.score,
        });

        self.send_stat(1, 0).await;

        // 更新统计
        let time_end = Local::now();
        let dur = time_end.signed_duration_since(time_start).num_milliseconds();

        self.stat.success += 1;
        self.stat.succ_dur += succ_dur as u64;
        self.stat.dur += dur as u64;

        debug!("[{}]{:?} detect ok.", item.index, item.file_name);
    }

    async fn save_align_img(&self, person_id: &str, content: &str) -> std::io::Result<()> {
        // /extdata/df_imgs/person/0037/003794dc-8283-4c1d-a262-661ed60c009b/003794dc-8283-4c1d-a262-661ed60c009b_341019.jpg

        let root = &self.ctx.cfg.df_imgs;

        // 现阶段，不知道faceid，暂时命名为1，后续重命名
        let face_id = 1_i64;
        let path = util::get_full_imgpath(root.as_str(), person_id, face_id);

        util::write_file_base64(path, content).await
    }
}

impl Service for DetectService {
    fn run(self, rx: Receiver<i64>) -> JoinHandle<()> {
        let mut svc = self;
        let mut exit_rx = rx;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    quit = exit_rx.recv() => {
                        if let Ok(100) = quit {
                            debug!("DetectService recv exit");
                            break;
                        }
                    },
                    item = svc.files.pop() => {
                        svc.do_queue(item).await;
                    }
                }
            }

            debug!("DetectService exit.");
        })
    }
}
