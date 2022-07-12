use std::sync::Arc;

use chrono::Local;
use deadqueue::unlimited::Queue;
use log::{debug, error, info};
use tokio::stream::StreamExt;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle as TkJoinHandle;

use cffc_base::model::img_file;

use crate::app_ctx::AppCtx;
use crate::dao::model::{CfFacetrack, CfPoi};
use crate::error::AppResult;
use crate::queue_item::{FtQI, FtQIPerson, QI};
use crate::services::face::face_search::FaceSearchWorker;
use crate::services::Service;

/// 多个worker，batch处理qi
/// mpsc 接收比对结果，检查报警，查询/更新数据库
/// 放入到后续队列中

pub struct FaceJudgeSvc {
    ctx: Arc<AppCtx>,
    queue: Arc<Queue<FtQI>>,
    out: Arc<Queue<QI>>,

    tx: UnboundedSender<FtQI>,
    rx: UnboundedReceiver<FtQI>,
}

impl FaceJudgeSvc {
    pub fn new(ctx: Arc<AppCtx>, queue: Arc<Queue<FtQI>>, out: Arc<Queue<QI>>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<FtQI>();

        FaceJudgeSvc {
            ctx,
            queue,
            tx,
            rx,
            out,
        }
    }

    // 查询数据库中的db列表
    fn load_dbs(&self) -> AppResult<Vec<(String, String, i32)>> {
        let dbs = self.ctx.dao.load_automatch_dbs()?;
        Ok(dbs)
    }

    fn fill_qi_person(&self, qi: &mut FtQIPerson, po: CfPoi) {
        let prefix = self.ctx.cfg.dfimg_url.as_str();
        qi.id = po.id;
        qi.name = po.name;

        qi.id_card = po.identity_card.map_or("".to_string(), |x| x);
        qi.gender = po.gender.map_or(0, |x| x as i64);
        qi.cover = po.cover.map_or(0, |x| x as i64);

        qi.cover_url = img_file::get_person_cover_url(prefix, po.poi_sid.as_str());
        qi.threshold = po.threshold as i64;

        match img_file::get_item_from_idscores(po.feature_ids.as_str()) {
            Ok(v) => {
                qi.imgs_url = v.iter().map(|x| {
                    img_file::get_person_img_url(prefix, qi.sid.as_str(), x.0)
                }).collect();
            }
            Err(e) => {
                error!("error, FaceJudgeSvc, get_item_from_idscores, {}", e);
            }
        };
    }


    /// 对比对的结果，查询数据库对应的person信息
    /// 判断报警情况
    /// 更新 facetrack表
    /// 放入后续队列中
    async fn process_item(&mut self, mut item: FtQI) {
        debug!("FaceJudgeSvc, recv item:{:?}", item.sid);

        // 查询person信息
        if let Some(ref mut qi) = item.match_poi {
            let ctx = self.ctx.clone();
            let poi_sid = qi.sid.clone();
            let po = tokio::task::spawn_blocking(move || {
                ctx.dao.load_poi_by_sid(poi_sid.as_str())
            }).await;

            if let Err(e) = po {
                error!("error, FaceJudgeSvc, process_item:{}, {:?}", item.sid, e);
            } else {
                let po = po.unwrap();
                if let Err(e) = po {
                    error!("error, FaceJudgeSvc, process_item:{}, {:?}", item.sid, e);
                } else {
                    let po = po.unwrap();
                    if po.is_none() {
                        error!("error, FaceJudgeSvc, {} matched poi:{}, poi not found.", item.sid, qi.sid);
                    } else {
                        let po = po.unwrap();
                        if qi.score >= po.threshold as i64 {
                            // 超过阈值，设置 item 已经 judged
                            item.face.judged = true;
                        }
                        self.fill_qi_person(qi, po);
                    }
                }
            }
        }

        // 判断报警
        if self.ctx.cfg.notify_proc.facetrack.wl_alarm {
            // 白名单报警模式
            item.face.alarmed = true;
            if let Some(ref mut person) = item.match_poi {
                if person.bw_flag == 2 && item.face.judged {
                    // 在白名单中，不报警
                    item.face.alarmed = false;
                }
            }
        } else {
            // 黑名单报警模式
            item.face.alarmed = false;
            if let Some(ref mut person) = item.match_poi {
                if person.bw_flag == 1 && item.face.judged {
                    // 在黑名单中，报警
                    item.face.alarmed = true;
                }
            }
        }

        // 更新db数据
        let ctx = self.ctx.clone();
        let now = Local::now();

        let facetrack = CfFacetrack {
            id: 0,
            ft_sid: item.sid.clone(),
            src_sid: "".to_string(),
            img_ids: "".to_string(),
            matched: match item.face.matched {
                true => Some(1),
                false => Some(0),
            },
            judged: match item.face.judged {
                true => Some(1),
                false => Some(0),
            },
            alarmed: match item.face.alarmed {
                true => Some(1),
                false => Some(0),
            },
            most_person: match item.match_poi {
                Some(ref v) => Some(v.sid.clone()),
                None => None,
            },
            most_score: match item.match_poi {
                Some(ref v) => Some(v.score as f64),
                None => None,
            },
            gender: None,
            age: None,
            glasses: None,
            direction: None,
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
            capture_time: now,
            gmt_create: now,
            gmt_modified: now,
        };

        let affect = tokio::task::spawn_blocking(move || {
            ctx.dao.upate_facetrack_for_judge(&facetrack)
        }).await;

        if let Err(e) = affect {
            error!("error, FaceJudgeSvc, process_item:{}, {:?}", item.sid, e);
        } else {
            let affect = affect.unwrap();
            if let Err(e) = affect {
                error!("error, FaceJudgeSvc, process_item:{}, {:?}", item.sid, e);
            } else {
                let affect = affect.unwrap();
                if affect == 1 {
                    debug!("FaceJudgeSvc, update facetrack, ok, {}", item.sid);
                } else {
                    debug!("error, FaceJudgeSvc, update facetrack, affect:{}, {}", affect, item.sid);
                }
            }
        }

        // 放入后续队列中
        debug!("FaceJudgeSvc, put ot next, {}", item.sid);
        self.out.push(QI::FT(item));
    }
}

impl Service for FaceJudgeSvc {
    fn run(self, rx: Receiver<i64>) -> TkJoinHandle<()> {
        let mut svc = self;
        let mut exit_rx = rx;

        // 加载 dbs
        let dbs = match svc.load_dbs() {
            Ok(v) => v,
            Err(e) => {
                error!("error, FaceJudgeSvc load_dbs(), {:?}", e);

                // 退出程序
                panic!("error, FaceJudgeSvc load_dbs(), {:?}", e);
            }
        };

        let mut services = Vec::new();
        let count = svc.ctx.cfg.notify_proc.search_worker as i64;
        for i in 0..count {
            let worker = FaceSearchWorker::new(i + 1, svc.ctx.clone(),
                                               svc.queue.clone(), dbs.clone(), svc.tx.clone());

            let rx = svc.ctx.exit_rx.clone();
            services.push(worker.run(rx));
        }

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    quit = exit_rx.next() => {
                        if let Some(100) = quit {
                            info!("FaceJudgeSvc recv exit");
                            break;
                        }
                    }
                    Some(item) = svc.rx.next() => {
                        svc.process_item(item).await;
                    }
                }
            }
            for h in services {
                let _ = h.await;
                info!("FaceJudgeSvc joined.")
            }

            info!("FaceJudgeSvc exit.");
        })
    }
}

