use std::sync::Arc;

use chrono::Local;
use deadqueue::unlimited::Queue;
use log::{debug, error, info};
use tokio::stream::StreamExt;
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle as TkJoinHandle;

use crate::app_ctx::AppCtx;
use crate::dao::model::{CfCartrack, CfCoi};
use crate::error::AppResult;
use crate::queue_item::{CtQI, CtQIPerson, QI};
use crate::services::Service;

pub struct CarJudgeSvc {
    ctx: Arc<AppCtx>,
    queue: Arc<Queue<CtQI>>,
    out: Arc<Queue<QI>>,
    groups: Vec<(String, String, i32)>,
}

impl CarJudgeSvc {
    pub fn new(ctx: Arc<AppCtx>, queue: Arc<Queue<CtQI>>, out: Arc<Queue<QI>>) -> Self {
        CarJudgeSvc {
            ctx,
            queue,
            out,
            groups: vec![],
        }
    }

    fn find_group(&self, sid: &str) -> Option<(String, String, i32)> {
        self.groups.iter().find(|&x| { x.0.eq(sid) }).map(|x| {
            (x.0.clone(), x.1.clone(), x.2)
        })
    }


    // 查询数据库中的db列表
    fn load_groups(&self) -> AppResult<Vec<(String, String, i32)>> {
        let groups = self.ctx.dao.load_coi_groups()?;
        Ok(groups)
    }

    fn fill_qi_person(&self, qi: &mut CtQI, po: CfCoi) {
        let group = match self.find_group(po.group_sid.as_str()) {
            Some(v) => v,
            None => {
                error!("error, CarJudgeSvc, can't find group:{} in cache", po.group_sid);
                return;
            }
        };

        let person = CtQIPerson {
            id: po.id,
            sid: po.sid,
            plate_content: po.plate_content,
            plate_type: po.plate_type,
            owner_name: po.owner_name,
            owner_phone: po.owner_phone,
            owner_address: po.owner_address,
            group_sid: group.0,
            group_name: group.1,
            bw_flag: group.2 as i64,
        };

        qi.match_coi = Some(person);
    }


    /// 对比对的结果，查询数据库对应的 coi 信息
    /// 判断报警情况
    /// 更新 cartrack表
    /// 放入后续队列中
    async fn process_item(&mut self, mut item: CtQI) {
        debug!("CarJudgeSvc, recv item:{:?}", item.sid);

        if let Some(ref plate) = item.car.plate {
            let ctx = self.ctx.clone();
            let plate_content = plate.content.clone();

            let po = tokio::task::spawn_blocking(move || {
                ctx.dao.load_coi_by_plate(plate_content.as_str())
            }).await;

            if let Err(e) = po {
                error!("error, CarJudgeSvc, process_item:{}, {:?}", item.sid, e);
            } else {
                let po = po.unwrap();
                if let Err(e) = po {
                    error!("error, CarJudgeSvc, process_item:{}, {:?}", item.sid, e);
                } else {
                    let po = po.unwrap();
                    if po.is_some() {
                        //
                        let po = po.unwrap();
                        self.fill_qi_person(&mut item, po);
                    }
                }
            }
        }

        // 判断报警
        if self.ctx.cfg.notify_proc.cartrack.wl_alarm {
            // 白名单报警模式
            item.car.alarmed = true;
            if let Some(ref mut person) = item.match_coi {
                if person.bw_flag == 2 {
                    // 在白名单中，不报警
                    item.car.alarmed = false;
                }
            }
        } else {
            // 黑名单报警模式
            item.car.alarmed = false;
            if let Some(ref mut person) = item.match_coi {
                if person.bw_flag == 1 {
                    // 在黑名单中，报警
                    item.car.alarmed = true;
                }
            }
        }

        // 更新db数据

        let now = Local::now();

        let cartrack = CfCartrack {
            id: 0,
            sid: item.sid.clone(),
            src_sid: "".to_string(),
            img_ids: "".to_string(),
            alarmed: match item.car.alarmed {
                true => 1,
                false => 0,
            },
            most_coi: match item.match_coi {
                Some(ref v) => Some(v.sid.clone()),
                None => None,
            },
            plate_judged: 0,
            vehicle_judged: 0,
            move_direct: 0,
            car_direct: None,
            plate_content: None,
            plate_confidence: None,
            plate_type: None,
            car_color: None,
            car_brand: None,
            car_top_series: None,
            car_series: None,
            car_top_type: None,
            car_mid_type: None,
            tag: None,
            flag: 0,
            obj_id: None,
            submit_id: None,
            submit_time: None,
            is_realtime: 0,
            capture_time: now,
            capture_ts: 0,
            capture_pts: 0,
            lane_num: 0,
            gmt_create: now,
            gmt_modified: now,
        };

        let ctx = self.ctx.clone();
        let affect = tokio::task::spawn_blocking(move || {
            ctx.dao.upate_cartrack_for_judge(&cartrack)
        }).await;

        if let Err(e) = affect {
            error!("error, CarJudgeSvc, process_item:{}, {:?}", item.sid, e);
        } else {
            let affect = affect.unwrap();
            if let Err(e) = affect {
                error!("error, CarJudgeSvc, process_item:{}, {:?}", item.sid, e);
            } else {
                let affect = affect.unwrap();
                if affect == 1 {
                    debug!("CarJudgeSvc, update cartrack, ok, {}", item.sid);
                } else {
                    debug!("error, CarJudgeSvc, update cartrack, affect:{}, {}", affect, item.sid);
                }
            }
        }

        // 放入后续队列中
        debug!("CarJudgeSvc, put ot next, {}", item.sid);
        self.out.push(QI::CT(Box::new(item)));
    }
}

impl Service for CarJudgeSvc {
    fn run(self, rx: Receiver<i64>) -> TkJoinHandle<()> {
        let mut svc = self;
        let mut exit_rx = rx;

        // 加载 groups
        let groups = match svc.load_groups() {
            Ok(v) => v,
            Err(e) => {
                error!("error, CarJudgeSvc load_groups(), {:?}", e);

                // 退出程序
                panic!("error, CarJudgeSvc load_groups(), {:?}", e);
            }
        };
        svc.groups = groups;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    quit = exit_rx.next() => {
                        if let Some(100) = quit {
                            info!("CarJudgeSvc recv exit");
                            break;
                        }
                    }
                    item = svc.queue.pop() => {
                        svc.process_item(item).await;
                    }
                }
            }

            info!("CarJudgeSvc exit.");
        })
    }
}

