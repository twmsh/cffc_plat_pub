use std::sync::Arc;
use std::time::Duration;

use log::{debug, info, error};
use tokio::sync::broadcast::{Receiver as BReceiver, Sender as BSender};
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use tokio::time;

use crate::cfg::{AppCtx, Stage, StageEvent};

use super::Service;
use chrono::prelude::*;

#[derive(Debug)]
pub struct StageStat {
    pub stages: Vec<Stage>,
}

impl StageStat {
    /// 创建，stage_count为stage个数, init_count 为第一个stage的count数,
    /// stage_id从0开始
    pub fn new(stage_count: usize, init_count: usize) -> Self {
        let mut stages = Vec::new();

        for x in 0..stage_count {
            let count = match x {
                0 => init_count,
                _ => 0,
            };

            stages.push(Stage {
                id: x,
                count,
                touch: 0,
                succ: 0,
                done: false,
            });
        }

        StageStat {
            stages,
        }
    }

    /// 处理事件，如果某个stage完成，需设置后一级的count
    /// 如果某一个的stage完成，但是succ是0，则后续stage不需要处理，后续stage的done设置成true
    pub fn process_event(&mut self, event: StageEvent) {
        let size = self.stages.len();
        if event.stage_id >= size {
            error!("error, stage_id({}) >= {}", event.stage_id, size);
            return;
        }

        let stage = self.stages.get_mut(event.stage_id).unwrap();
        stage.succ += event.succ;
        stage.touch += event.succ + event.fail;

        /*
        if stage.touch > stage.count {
            error!("error, {}, stage.touch({}) > stage.count({})", stage.id, stage.touch, stage.count);
            return;
        }*/

        if stage.touch == stage.count {
            // 根据当前stage的succ，设置下一个stage的count
            stage.done = true;
            let next_id = stage.id + 1;
            let next_count = stage.succ;

            if let Some(v) = self.stages.get_mut(next_id) {
                v.count = next_count;
                debug!("set next stage, id:{}, count:{}", v.id, v.count);
            }
        }
    }

    /// 所有stage 是否处理完成
    /// 从前往后检查，done为true,且succ=0，则短路，直接返回true
    /// 否则有false，返回flase
    pub fn is_done(&self) -> bool {
        let size = self.stages.len();
        let mut done = true;

        for i in 0..size {
            let stage = self.stages.get(i).unwrap();
            done = done && stage.done;

            if !done {
                return false;
            }

            if done && stage.succ == 0 {
                return true;
            }
        }

        done
    }

    /// 任务总数
    pub fn task_count(&self) -> usize {
        self.stages.first().map_or(0, |x| x.count)
    }

    /// 成功数
    pub fn task_succ(&self) -> usize {
        self.stages.last().map_or(0, |x| x.succ)
    }
}

//------------------------------
pub struct StageStatService {
    pub ctx: Arc<AppCtx>,
    pub stage_stat: StageStat,

    /// 接收 统计信息
    pub rx: Receiver<StageEvent>,

    /// 向退出通道发送消息
    pub sender: BSender<i64>,

    pub start_time: DateTime<Local>,
    pub end_time: Option<DateTime<Local>>,
}

impl StageStatService {
    /// 新建
    /// 一个3个stage: detect, create_persons, save_db
    pub fn new(ctx: Arc<AppCtx>, init_count: usize, rx: Receiver<StageEvent>) -> Self {
        let stage_stat = StageStat::new(3, init_count);
        StageStatService {
            stage_stat,
            rx,
            sender: ctx.exit_tx.clone(),
            ctx,
            start_time: Local::now(),
            end_time: None,
        }
    }

    pub fn send_exit(&self) {
        match self.sender.send(100) {
            Ok(v) => {
                debug!("StageStatService send exit channel, {}", v);
            }
            Err(e) => {
                error!("error, StageStatService send exit fail, {:?}", e);
            }
        };
    }

    pub fn print_stat(&self) {
        info!("{:?}", self.stage_stat.stages);
    }

    pub fn print_ops(&self) {
        let end_time = match self.end_time {
            Some(v) => v,
            None => {
                return;
            }
        };

        let dur = end_time.signed_duration_since(self.start_time).num_milliseconds();
        let count = self.stage_stat.task_count();
        let succ = self.stage_stat.task_succ();
        let tps: f64 = count as f64 / (dur as f64 / 1000_f64);

        info!("task: {}/{}, use: {} mills, tps: {}", succ, count, dur, tps);
    }
}

impl Service for StageStatService {
    fn run(self, rx: BReceiver<i64>) -> JoinHandle<()> {
        let mut svc = self;
        let mut exit_rx = rx;

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(1));


            loop {
                tokio::select! {
                    quit = exit_rx.recv() => {
                        if let Ok(100) = quit {
                            debug!("StageStatService recv exit");
                            break;
                        }
                    },
                    _ = interval.tick() => {
                        svc.print_stat();
                    },
                    event = svc.rx.recv() => {
                        if let Some(e) = event {
                            svc.stage_stat.process_event(e);
                            if svc.stage_stat.is_done() {
                                debug!("StageStatService, all stages done, will exit");
                                break;
                            }
                        }else{
                            debug!("StageStatService, channel closed, will exit");
                        }
                    }
                }
            }
            svc.end_time = Some(Local::now());

            if svc.stage_stat.is_done() {
                // 退出通道
                svc.send_exit();
            } else {
                error!("error, DetectService, channel closed, but stages not done")
            }
            svc.print_stat();
            svc.print_ops();

            debug!("DetectService exit.");
        })
    }
}
