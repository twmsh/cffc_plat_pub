use std::ffi::OsString;
use std::sync::Arc;

use chrono::prelude::*;
use deadqueue::unlimited::Queue;
use log::{debug, error};

use tokio::sync::broadcast::Receiver;
use tokio::task::JoinHandle;

use cffc_base::api::bm_api::{ApiFeatureQuality, RecognitionApi};
use crate::cfg::{AppCtx, CreateItem, FeaItem, TaskStat, StageEvent};

use crate::util;
use super::Service;
use tokio::sync::mpsc::{Sender as MSender};

pub struct CreatePersonService {
    pub ctx: Arc<AppCtx>,
    pub stat: TaskStat,
    pub queue: Arc<Queue<FeaItem>>,
    pub out: Arc<Queue<CreateItem>>,

    pub api: RecognitionApi,

    pub stage_id: usize,
    /// workder id
    pub stage_wid: usize,

    pub stat_sender: MSender<StageEvent>,
}

/// 从队列中取出feaitem，多线程，成批提交（调用api)
/// 通过api，获得face_id，放到后续队列中
impl CreatePersonService {
    pub fn new(ctx: Arc<AppCtx>, queue: Arc<Queue<FeaItem>>,
               out: Arc<Queue<CreateItem>>, stage_wid: usize) -> Self {
        let api = RecognitionApi::new(ctx.cfg.recog.url.as_str());

        CreatePersonService {
            stat_sender: ctx.stat_tx.clone(),
            ctx,
            stat: Default::default(),
            queue,
            out,
            api,
            stage_id: 1,
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
            error!("error, CreatePersonService.send_stat, {:?}", e);
        }
    }

    /// 队列中数据时，一次最多取出limit数据, 没有数据时候阻塞，取一条数据返回
    async fn drain_it(&mut self) -> Vec<FeaItem> {
        let max = self.ctx.cfg.create_batch;
        let mut size = 0_u64;
        let mut list: Vec<FeaItem> = Vec::new();

        while let Some(v) = self.queue.try_pop() {
            list.push(v);
            size += 1;
            if size == max {
                break;
            }
        }

        if list.is_empty() {
            let v = self.queue.pop().await;
            list.push(v);
        }
        list
    }

    fn find_index(&self, person_id: &str, list: &[FeaItem]) -> Option<(u32, OsString, f64)> {
        for v in list {
            if v.person_id == person_id {
                return Some((v.index, v.file_name.clone(), v.score));
            }
        }

        None
    }

    async fn rename_img(&self, person_id: &str, face_id: i64) -> std::io::Result<()> {
        let root = &self.ctx.cfg.df_imgs;
        let src_path = util::get_full_imgpath(root.as_str(), person_id, 1);
        let dst_path = util::get_full_imgpath(root.as_str(), person_id, face_id);

        tokio::fs::rename(src_path, dst_path).await
    }


    async fn do_list(&mut self, list: Vec<FeaItem>) {
        let time_start = Local::now();
        let db = self.ctx.cfg.recog.db_sid.clone();

        let mut ids: Vec<String> = Vec::new();
        let mut features_list: Vec<Vec<ApiFeatureQuality>> = Vec::new();

        let _: Vec<()> = list.iter().map(|x| {
            ids.push(x.person_id.clone());
            features_list.push(vec![ApiFeatureQuality {
                feature: x.fea.clone(),
                quality: 1.0,
            }]);
        }).collect();


        let res = match self.api.create_persons(db, ids.clone(), features_list).await {
            Ok(v) => v,
            Err(e) => {
                error!("error, create_persons: [{:?}], {:?}", ids, e);
                self.send_stat(0, list.len()).await;
                return;
            }
        };

        let time_2 = Local::now();
        let succ_dur = time_2.signed_duration_since(time_start).num_milliseconds();

        if res.code != 0 {
            error!("error, create_persons: [{:?}], return code:{}, msg:{}"
                   , ids, res.code, res.msg);
            self.send_stat(0, list.len()).await;
            return;
        }

        let res_persons = match res.persons {
            Some(v) => v,
            None => {
                error!("error, create_persons: [{:?}], return persons is null"
                       , ids);
                self.send_stat(0, list.len()).await;
                return;
            }
        };
        // 提交的ids和返回的person数量不一致
        if res_persons.len() != ids.len() {
            error!("error, create_persons, persons len:{} not equal ids len{}"
                   , res_persons.len(), ids.len());
            self.send_stat(0, list.len()).await;
            return;
        }

        // 放入后续队列中
        for v in res_persons {
            let face_id = match v.faces.get(0) {
                Some(v) => *v,
                None => {
                    error!("error, create_persons, {} faces is null"
                           , v.id);
                    self.send_stat(0, 1).await;
                    continue;
                }
            };

            let index_tup = match self.find_index(v.id.as_str(), &list) {
                Some(v) => v,
                None => {
                    error!("error, can't find FeaItem for {}"
                           , v.id);
                    self.send_stat(0, 1).await;
                    continue;
                }
            };

            //修改图片文件名
            match self.rename_img(v.id.as_str(), face_id).await {
                Ok(_) => {}
                Err(e) => {
                    error!("error, img rename fail:[{}, {}], {:?}"
                           , v.id, face_id, e);
                    self.send_stat(0, 1).await;
                    continue;
                }
            };


            let item = CreateItem {
                index: index_tup.0,
                file_name: index_tup.1,
                person_id: v.id,
                face_id,
                score: index_tup.2,
            };
            debug!("push CreateItem: {}, {}, {:?}", item.index, item.person_id, item.file_name);
            self.out.push(item);
            self.send_stat(1, 0).await;
        }


        let time_end = Local::now();
        let dur = time_end.signed_duration_since(time_start).num_milliseconds();

        self.stat.success += 1;
        self.stat.succ_dur += succ_dur as u64;
        self.stat.dur += dur as u64;
    }
}

impl Service for CreatePersonService {
    fn run(self, rx: Receiver<i64>) -> JoinHandle<()> {
        let mut svc = self;
        let mut exit_rx = rx;

        tokio::spawn(async move {
            loop {
                // 检查退出信号

                // 处理队列的数据，成批处理
                tokio::select! {
                    quit = exit_rx.recv() => {
                        if let Ok(100) = quit {
                            debug!("CreatePersonService recv exit");
                            break;
                        }
                    },
                    list = svc.drain_it() => {
                        debug!("CreatePersonService drain_it, len:{}",list.len());
                        svc.do_list(list).await;
                    }
                }
            }
            debug!("CreatePersonService exit.");
        })
    }
}