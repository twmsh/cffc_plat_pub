use std::sync::Arc;
use chrono::prelude::*;
use deadqueue::unlimited::Queue;
use log::{debug, error, info};
use tokio::stream::StreamExt;
use tokio::sync::mpsc::{UnboundedSender};
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle as TkJoinHandle;

use cffc_base::api::bm_api::{ApiFeatureQuality, RecognitionApi, SearchResPerson};
use cffc_base::util::utils;

use crate::app_ctx::AppCtx;
use crate::queue_item::{FtQI, FtQIPerson};
use crate::services::Service;

pub struct FaceSearchWorker {
    num: i64,
    ctx: Arc<AppCtx>,
    queue: Arc<Queue<FtQI>>,

    skip_search: bool,

    /// sid,name,bw_flag
    dbs: Vec<(String, String, i32)>,
    tx: UnboundedSender<FtQI>,
    api: RecognitionApi,

}

impl FaceSearchWorker {
    pub fn new(num: i64, ctx: Arc<AppCtx>, queue: Arc<Queue<FtQI>>, dbs: Vec<(String, String, i32)>, tx: UnboundedSender<FtQI>) -> Self {
        let api = RecognitionApi::new(ctx.cfg.api.recg_url.as_str());
        let skip_search = ctx.cfg.notify_proc.skip_search;

        FaceSearchWorker {
            num,
            ctx,
            api,
            queue,
            skip_search,
            dbs,
            tx,
        }
    }

    async fn pop_batch(&mut self) -> Vec<FtQI> {
        let max = self.ctx.cfg.notify_proc.search_batch as usize;
        utils::pop_queue_batch(&self.queue, max).await
    }


    fn find_db(&self, db_sid: &str) -> Option<(String, String, i32)> {
        self.dbs.iter().find(|&x| { x.0.eq(db_sid) }).map(|x| {
            (x.0.clone(), x.1.clone(), x.2)
        })
    }

    fn fill_with_matchinfo(&self, items: &mut Vec<FtQI>, persons: &Option<Vec<Vec<SearchResPerson>>>) {
        let persons = match persons.as_ref() {
            Some(v) => v,
            None => {
                error!("error, FaceSearchWorker[{}], search result persons is none", self.num);
                return;
            }
        };

        if persons.len() != items.len() {
            error!("error, FaceSearchWorker[{}], items len:{} not equal result persons:{}", self.num, persons.len(), items.len());
            return;
        }

        let count = items.len();
        for i in 0..count {
            let item = items.get_mut(i).unwrap();

            // 设置 已经执行match
            item.face.matched = true;

            // 取 top 1
            let person = match persons.get(i).unwrap().get(0) {
                Some(v) => v,
                None => {
                    debug!("FaceSearchWorker[{}], search facetrack:{}, has no match", self.num, item.sid);
                    continue;
                }
            };

            let db = match self.find_db(person.db.as_str()) {
                Some(v) => v,
                None => {
                    error!("error, FaceSearchWorker[{}], can't find db:{} in cache", self.num, person.db);
                    continue;
                }
            };

            item.match_poi = Some(FtQIPerson {
                id: 0,
                sid: person.id.clone(),
                name: "".to_string(),
                id_card: "".to_string(),
                gender: 0,
                cover: 0,
                cover_url: "".to_string(),
                imgs_url: vec![],
                threshold: 0,
                score: person.score,
                db_sid: db.0,
                db_name: db.1,
                bw_flag: db.2 as i64,
            });

            debug!("FaceSearchWorker[{}], facetrack:{}, matched:{}, score:{}", self.num, item.sid, person.id, person.score);
        }
    }

    /// api 比对搜索，(有特征值, 并且dbs不为空)
    /// 无论处理成功或失败，都提交到mpsc中
    async fn process_batch(&mut self, mut items: Vec<FtQI>) {
        let tops = vec![1_i64];
        let thresholds = vec![0_i64];
        let dbs: Vec<String> = self.dbs.iter().map(|x| x.0.clone()).collect();
        let mut persons = Vec::new();

        debug!("FaceSearchWorker[{}], process_batch: {}", self.num, items.len());

        items.iter_mut().for_each(|x| {
            let mut feas = Vec::new();
            x.face.faces.iter_mut().for_each(|f| {
                if let Some(ref feature) = f.feature {
                    feas.push(ApiFeatureQuality {
                        feature: feature.clone(),
                        quality: f.quality,
                    });
                }
                // 清除 feature
                f.feature.take();
            });

            if !feas.is_empty() {
                // 不为空，才加入到 最后的比对中
                persons.push(feas);
            }
        });

        if self.skip_search || dbs.is_empty() || persons.len() != items.len() {
            // dbs 为空，或者 items中存在没有特征值的item，则不进行比对
            debug!("FaceSearchWorker[{}], skip search", self.num);
        } else {
            let ts_start = Local::now();
            let search_res = self.api.search(dbs, tops, thresholds, persons).await;
            let ts_use = Local::now().signed_duration_since(ts_start);
            debug!("FaceSearchWorker[{}], search api use: {} ms, batch size:{}", self.num, ts_use.num_milliseconds(), items.len());

            match search_res {
                Ok(res) => {
                    if res.code == 0 {
                        // 填充 比对结果
                        self.fill_with_matchinfo(&mut items, &res.persons);
                    } else {
                        error!("error, FaceSearchWorker[{}], search, code:{}, msg:{}", self.num, res.code, res.msg);
                    }
                }
                Err(e) => {
                    error!("error, FaceSearchWorker[{}], search, {:?}", self.num, e);
                }
            }
        }

        // 放入mpsc中
        for v in items {
            match self.tx.send(v) {
                Ok(_) => {}
                Err(e) => {
                    error!("error, FaceSearchWorker[{}], tx.send, {:?}", self.num, e);
                }
            };
        }
    }
}

impl Service for FaceSearchWorker {
    fn run(self, rx: Receiver<i64>) -> TkJoinHandle<()> {
        let mut svc = self;
        let mut exit_rx = rx;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    quit = exit_rx.next() => {
                        if let Some(100) = quit {
                            info!("FaceSearchWorker[{}] recv exit",svc.num);
                            break;
                        }
                    }
                    items = svc.pop_batch() => {
                        svc.process_batch(items).await;
                    }
                }
            }
            info!("FaceSearchWorker[{}] exit", svc.num);
        })
    }
}

