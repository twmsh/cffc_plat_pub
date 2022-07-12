use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use deadqueue::unlimited::Queue;
use log::{debug, info};
use tokio::stream::StreamExt;
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle as TkJoinHandle;


use crate::app_ctx::AppCtx;

use crate::queue_item::QI;

use super::Service;

pub struct EntBusSvc {
    _ctx: Arc<AppCtx>,
    queue: Arc<Queue<QI>>,
    out_queues: RwLock<HashMap<String, Arc<Queue<QI>>>>,
}

impl EntBusSvc {
    pub fn new(_ctx: Arc<AppCtx>, queue: Arc<Queue<QI>>) -> Self {
        EntBusSvc {
            _ctx,
            queue,
            out_queues: RwLock::default(),
        }
    }

    pub fn get_queue(&self, name: &str) -> Arc<Queue<QI>> {
        let mut lock = self.out_queues.write().unwrap();
        let queue = lock.entry(name.to_string()).or_insert_with(|| Arc::new(Queue::new()));
        queue.clone()
    }

    async fn process_item(&mut self, item: QI) {
        debug!("EntBusSvc, process_item, {}, type: {}", item.get_sid(), item.get_type());

        let lock = self.out_queues.read().unwrap();

        for (_k, v) in lock.iter() {
            v.push(item.clone());
        }
    }
}

impl Service for EntBusSvc {
    fn run(self, rx: Receiver<i64>) -> TkJoinHandle<()> {
        let mut svc = self;
        let mut exit_rx = rx;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    quit = exit_rx.next() => {
                        if let Some(100) = quit {
                            info!("EntBusSvc recv exit");
                            break;
                        }
                    }
                    item = svc.queue.pop() => {
                        svc.process_item(item).await;
                    }
                }
            }
            info!("EntBusSvc exit.");
        })
    }
}
