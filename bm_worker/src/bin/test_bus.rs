#![allow(dead_code)]
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use deadqueue::unlimited::Queue;
use bm_worker::queue_item::QI;

pub struct Item {
    pub id: i64,
    pub name: String,
}

pub struct EBus {
    queue: Arc<Queue<QI>>,
    out_queues: RwLock<HashMap<String, Arc<Queue<QI>>>>,
}

impl EBus {
    pub fn new(input: Arc<Queue<QI>>) -> Self {
        EBus {
            queue: input,
            out_queues: RwLock::default(),
        }
    }

    pub fn get_queue(&self, name: &str) -> Arc<Queue<QI>> {
        let mut lock = self.out_queues.write().unwrap();
        let queue = lock.entry(name.to_string()).or_insert_with(|| Arc::new(Queue::new()));
        queue.clone()
    }

    pub fn process_item(&self, item: QI) {
        let lock = self.out_queues.read().unwrap();

        for (_k, v) in lock.iter() {
            v.push(item.clone());
        }
    }
}

pub fn main() {}