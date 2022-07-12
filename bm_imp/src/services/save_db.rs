use std::sync::Arc;

use chrono::prelude::*;
use deadqueue::unlimited::Queue;
use log::{debug, error};
use regex::Regex;
use rusqlite::Connection;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender as MSender;
use tokio::task::JoinHandle;

use crate::cfg::{AppCtx, CreateItem, ImpPersonInfo, StageEvent, TaskStat};
use bm_worker::dao::model::CfPoi;

use super::Service;

pub struct SaveDbService {
    pub ctx: Arc<AppCtx>,
    pub stat: TaskStat,
    pub queue: Arc<Queue<CreateItem>>,
    pub regex: Regex,

    pub stage_id: usize,
    /// workder id
    pub stage_wid: usize,

    pub stat_sender: MSender<StageEvent>,
}

impl SaveDbService {
    pub fn new(ctx: Arc<AppCtx>, queue: Arc<Queue<CreateItem>>, regex: Regex, stage_wid: usize) -> Self {
        SaveDbService {
            stat_sender: ctx.stat_tx.clone(),
            ctx,
            stat: Default::default(),
            queue,
            regex,
            stage_id: 2,
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
            error!("error, SaveDbService.send_stat, {:?}", e);
        }
    }


    async fn drain_it(&mut self) -> Vec<CreateItem> {
        let max = self.ctx.cfg.save_batch;
        let mut size = 0_u64;
        let mut list: Vec<CreateItem> = Vec::new();

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

    async fn do_save(&mut self, persons: Vec<ImpPersonInfo>) -> usize {
        let ctx = self.ctx.clone();

        let h = tokio::task::spawn_blocking(move || {
            let mut guard = ctx.dao.lock().unwrap();
            let conn = guard.conn.get_mut();
            db_save_batch(conn, persons, ctx.cfg.recog.db_sid.clone())
        });

        match h.await {
            Ok(v) => v,
            Err(e) => {
                error!("error, do_save, {:?}", e);
                0_usize
            }
        }
    }

    /// 批次保存
    async fn do_list(&mut self, list: Vec<CreateItem>) {
        let time_start = Local::now();


        //转换成 ImpPersonInfo
        let props = &self.ctx.cfg.imp.props;

        let mut persons: Vec<ImpPersonInfo> = Vec::new();
        let _: Vec<()> = list.iter().map(|x| {
            match ImpPersonInfo::from_filename(x.file_name.as_os_str(),
                                               &self.regex, props) {
                Ok(mut v) => {
                    v.index = x.index;
                    v.person_id = x.person_id.clone();
                    v.face_id = x.face_id;
                    v.score = x.score;

                    v.imp_tag = Some(self.ctx.cfg.imp.imp_tag.clone());
                    v.threshold = self.ctx.cfg.imp.threshold as i32;

                    persons.push(v);
                }
                Err(e) => {
                    error!("error, from_filename:{:?}, {:?}", x.file_name, e);
                }
            };
        }).collect();

        let saved = self.do_save(persons).await;
        self.send_stat(saved, list.len() - saved).await;

        let time_end = Local::now();
        let dur = time_end.signed_duration_since(time_start).num_milliseconds();

        self.stat.success += saved as u64;
        self.stat.succ_dur += dur as u64;
        self.stat.dur += dur as u64;
    }
}

impl Service for SaveDbService {
    fn run(self, rx: Receiver<i64>) -> JoinHandle<()> {
        let mut svc = self;
        let mut exit_rx = rx;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    quit = exit_rx.recv() => {
                        if let Ok(100) = quit {
                            debug!("SaveDbService recv exit");
                            break;
                        }
                    },
                    list = svc.drain_it() => {
                        debug!("SaveDbService drain_it, len:{}",list.len());
                        svc.do_list(list).await;
                    }
                }
            }
            debug!("SaveDbService exit.");
        }
        )
    }
}

//----------------
fn db_save_batch(conn: &mut Connection, persons: Vec<ImpPersonInfo>, db_sid: String) -> usize {
    let mut succ = 0_usize;

    let tx = match conn.transaction() {
        Ok(v) => v,
        Err(e) => {
            error!("error, conn.transaction, {:?}", e);
            return 0_usize;
        }
    };

    for info in persons {
        let now = Local::now();
        let poi = CfPoi {
            id: 0,
            poi_sid: info.person_id,
            db_sid: db_sid.clone(),
            name: info.name,
            gender: Some(info.gender as i32),
            identity_card: Some(info.identity_card),
            threshold: info.threshold,
            tp_id: None,
            feature_ids: format!("{}:{}", info.face_id, info.score),
            cover: Some(0),
            tag: None,
            imp_tag: info.imp_tag,
            memo: Some(info.memo),
            flag: Some(0),
            gmt_create: now,
            gmt_modified: now,
        };

        match poi_insert(&poi, &tx) {
            Ok(_) => {
                succ += 1;
                debug!("[{}], {} saved db, ok", info.index, poi.poi_sid);
            }
            Err(e) => {
                error!("error, [{}], {} saved fail, {:?}", info.index, poi.poi_sid, e);
            }
        };
    }

    match tx.commit() {
        Ok(_) => {}
        Err(e) => {
            error!("error, transaction.commit fail, {:?}", e);
        }
    };
    succ
}

//------------------------------------------------------------------
use rusqlite::{Transaction, params};
use cffc_base::db::dbop::Error as DbopError;

pub fn poi_insert(po: &CfPoi, con: &Transaction) -> Result<i64, DbopError> {
    let sql = "insert into cf_poi(poi_sid,db_sid,name,gender,identity_card,threshold,tp_id,feature_ids,cover,tag,imp_tag,memo,flag,gmt_create,gmt_modified) values(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)";
    let mut stmt = con.prepare(sql)?;
    let _affect = stmt.execute(params![po.poi_sid,po.db_sid,po.name,po.gender,po.identity_card,po.threshold,po.tp_id,po.feature_ids,po.cover,po.tag,po.imp_tag,po.memo,po.flag,po.gmt_create,po.gmt_modified])?;

    let id = con.last_insert_rowid();
    Ok(id)
}
