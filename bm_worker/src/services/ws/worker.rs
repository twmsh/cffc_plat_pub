use std::sync::Arc;
use std::thread;

use actix::prelude::*;
use deadqueue::unlimited::Queue;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use tokio::runtime;
use tokio::stream::StreamExt;

use cffc_base::util::utils;

use crate::app_ctx::AppCtx;
use crate::error::AppResult;
use crate::queue_item::{CtQI, FtQI, QI};
use crate::services::ws::{DeliverMessage, QiMessage, RegisterMessage, SessionConnect};

const TRACK_ROOM: &str = "track";

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WsMsgStat {
    pub total_face_count: i64,
    pub total_face_alarm: i64,

    pub total_car_count: i64,
    pub total_car_alarm: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WsMsg {
    pub stat: WsMsgStat,
    pub track: Vec<QI>,
}

pub struct WsWorker {
    ctx: Arc<AppCtx>,
    queue: Arc<Queue<QI>>,
    agent_addr: Option<Recipient<DeliverMessage>>,

    //----
    track_snap: TrackSnap,
}

impl Actor for WsWorker {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // 启动事件队列处理
        let addr = ctx.address();
        self.start_queue_loop(addr);
    }
}

impl WsWorker {
    pub fn new(ctx: Arc<AppCtx>, queue: Arc<Queue<QI>>) -> Self {
        let batch = ctx.cfg.ws.batch;
        WsWorker {
            ctx,
            queue,
            agent_addr: None,
            track_snap: TrackSnap::new(batch),
        }
    }

    fn deliver_snap_msg(&self, id: usize) {
        let items = self.track_snap.buf.clone();

        let ws_msg = WsMsg {
            stat: self.track_snap.stat.clone(),
            track: items,
        };

        let content = serde_json::to_string(&ws_msg);
        if let Err(e) = content {
            error!("error, WsWorker, serde_json::to_string, {:?}", e);
            return;
        }
        let content = content.unwrap();

        self.deliver_msg(DeliverMessage {
            msg: content,
            room: TRACK_ROOM.to_string(),
            id,
        });
    }

    fn deliver_broadcast_msg(&self, msg: String) {
        self.deliver_msg(DeliverMessage {
            msg,
            room: TRACK_ROOM.to_string(),
            id: 0,
        });
    }

    fn deliver_msg(&self, msg: DeliverMessage) {
        if let Some(ref addr) = self.agent_addr {
            if let Err(e) = addr.do_send(msg) {
                error!("error, WsWorker, do_send, {:?}", e);
            }
        }
    }

    fn start_queue_loop(&self, addr: Addr<WsWorker>) {
        let mut exit_rx = self.ctx.exit_rx.clone();
        let queue = self.queue.clone();
        let batch = 10_usize;

        let _h = thread::spawn(move || {
            let mut basic_rt = runtime::Builder::new()
                .basic_scheduler()
                .build().unwrap();

            basic_rt.block_on(async {
                debug!("WsWorker, start_queue_loop.");
                loop {
                    tokio::select! {
                    quit = exit_rx.next() => {
                        if let Some(100) = quit {
                            info!("WsWorker recv exit");
                            break;
                        }
                    }
                    item = pop_batch(&queue,batch) => {
                        addr.do_send(QiMessage(item));
                    }
                }
                }
                debug!("WsWorker, start_queue_loop, exit.");
            });
        });
    }

    /// 加载历史数据
    ///
    /// list1，list2 是时间倒序
    // 返回list，是合并后的，时间正序的
    pub fn load(&mut self) -> AppResult<()> {
        let limit = self.ctx.cfg.ws.batch;
        let prefix = &self.ctx.cfg.dfimg_url;

        let total_face = self.ctx.dao.get_facetrack_count()?.unwrap();
        let total_face_alarm = self.ctx.dao.get_facetrack_alarm_count()?.unwrap();
        let total_car = self.ctx.dao.get_cartrack_count()?.unwrap();
        let total_car_alarm = self.ctx.dao.get_cartrack_alarm_count()?.unwrap();

        let facetrack_list = self.ctx.dao.load_latest_facetrack_list(limit as i64)?;
        let cartrack_list = self.ctx.dao.load_latest_cartrack_list(limit as i64)?;

        self.track_snap.stat.total_face_count = total_face;
        self.track_snap.stat.total_face_alarm = total_face_alarm;
        self.track_snap.stat.total_car_count = total_car;
        self.track_snap.stat.total_car_alarm = total_car_alarm;

        let camera_list = self.ctx.web_dao.get_all_sourcelist()?;
        let db_list = self.ctx.web_dao.get_dfdb_list()?;
        let car_group_list = self.ctx.web_dao.get_coigroup_list()?;

        let mut qi_list = Vec::new();

        //人脸记录
        for v in facetrack_list.iter() {
            let camera = camera_list.iter().find_map(|x| {
                if x.src_sid.eq(&v.src_sid) {
                    Some(x)
                } else {
                    None
                }
            });

            let match_poi = match v.most_person {
                Some(ref sid) => {
                    self.ctx.dao.load_poi_by_sid(sid)?
                }
                None => None,
            };

            let qi_ft = FtQI::from_po(prefix, v, camera, &db_list, match_poi)?;
            qi_list.push(QI::FT(qi_ft));
        }

        // 车辆记录
        for v in cartrack_list.iter() {
            let camera = camera_list.iter().find_map(|x| {
                if x.src_sid.eq(&v.src_sid) {
                    Some(x)
                } else {
                    None
                }
            });

            let match_coi = match v.plate_content {
                Some(ref plate) => {
                    self.ctx.dao.load_coi_by_plate(plate)?
                }
                None => None,
            };

            let qi_ct = CtQI::from_po(prefix, v, camera, &car_group_list, match_coi)?;
            qi_list.push(QI::CT(Box::new(qi_ct)));
        }

        // 排序
        qi_list.sort_by_key(|x| {
            match x {
                QI::FT(v) => v.face.ts,
                QI::CT(v) => v.car.ts,
            }
        });

        // 去掉多余的
        if qi_list.len() > limit {
            let remove = qi_list.len() - limit;
            qi_list.drain(0..remove);
        }

        debug!("WsWorker, load: {} tracks", qi_list.len());
        self.track_snap.buf = qi_list;

        Ok(())
    }
}


async fn pop_batch(queue: &Arc<Queue<QI>>, max: usize) -> Vec<QI> {
    utils::pop_queue_batch(queue, max).await
}


impl Handler<SessionConnect> for WsWorker {
    type Result = usize;

    /// ws连接上来时候，发送快照数据下去
    fn handle(&mut self, msg: SessionConnect, _ctx: &mut Context<Self>) -> Self::Result {
        self.deliver_snap_msg(msg.id);
        msg.id
    }
}

impl Handler<RegisterMessage> for WsWorker {
    type Result = ();

    /// 记录 agent的地址
    fn handle(&mut self, msg: RegisterMessage, _ctx: &mut Context<Self>) -> Self::Result {
        debug!("WsWorker, handle RegisterMessage");
        self.agent_addr = Some(msg.addr);
    }
}

impl Handler<QiMessage> for WsWorker {
    type Result = ();

    fn handle(&mut self, msg: QiMessage, _ctx: &mut Context<Self>) -> Self::Result {
        let items = self.track_snap.append(msg.0);

        let ws_msg = WsMsg {
            stat: self.track_snap.stat.clone(),
            track: items,
        };

        let content = serde_json::to_string(&ws_msg);
        if let Err(e) = content {
            error!("error, WsWorker, serde_json::to_string, {:?}", e);
            return;
        }

        self.deliver_broadcast_msg(content.unwrap());
    }
}


//--------------------
#[derive(Debug, Clone)]
pub struct TrackSnap {
    pub cap: usize,
    pub buf: Vec<QI>,

    pub stat: WsMsgStat,
}

impl TrackSnap {
    pub fn new(cap: usize) -> Self {
        TrackSnap {
            cap,
            buf: Vec::new(),
            stat: Default::default(),
        }
    }

    fn stat_it(&mut self, item: &QI) {
        match item {
            QI::FT(v) => {
                self.stat.total_face_count += 1;
                if v.face.alarmed {
                    self.stat.total_face_alarm += 1;
                }
            }
            QI::CT(v) => {
                self.stat.total_car_count += 1;
                if v.car.alarmed {
                    self.stat.total_car_alarm += 1;
                }
            }
        }
    }

    fn add(&mut self, item: QI) {
        self.stat_it(&item);

        if self.buf.len() < self.cap {
            self.buf.push(item);
        } else {
            self.buf.remove(0);
            self.buf.push(item);
        }
    }

    /// 更新快照
    /// 返回新增的数据，如果数据超过cap，只返回最新的cap条数据
    pub fn append(&mut self, list: Vec<QI>) -> Vec<QI> {
        let mut rst = Vec::new();
        let index = if list.len() > self.cap {
            list.len() - self.cap
        } else {
            0
        };
        list.iter().enumerate().for_each(|(i, v)| {
            if i >= index {
                rst.push(v.clone());
            }
        });


        for v in list {
            self.add(v);
        }

        rst
    }
}
