#![allow(clippy::mutex_atomic)]

use std::sync::{atomic::{AtomicBool, Ordering}, Condvar, Mutex};

use crossbeam::queue::SegQueue;

// spsc 模式

pub struct IntrSegQueue<T> {
    queue: SegQueue<T>,
    mux: Mutex<bool>,
    // 是否要阻塞
    condvar: Condvar,
    closed: AtomicBool, // 队列是否关闭
}

impl<T> Default for IntrSegQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> IntrSegQueue<T> {
    pub fn new() -> IntrSegQueue<T> {
        IntrSegQueue {
            queue: SegQueue::new(),
            mux: Mutex::new(true),
            condvar: Condvar::new(),
            closed: AtomicBool::new(false),
        }
    }

    /**
    队列关闭时候，返回 false
    无阻塞
    */
    pub fn put(&self, v: T) -> bool {
        if self.closed.load(Ordering::Acquire)
        {
            return false;
        }
        self.queue.push(v);

        let mut gurad = self.mux.lock().unwrap();
        let empty = self.queue.is_empty();
        if !empty {
            *gurad = false;
            self.condvar.notify_one();
        }

        true
    }

    /**
    队列关闭时候，返回None
    无数据时候，阻塞
    */
    pub fn get(&self) -> Option<T> {
        if self.closed.load(Ordering::Acquire)
        {
            return None;
        }

        if let Some(v) = self.queue.pop() {
            return Some(v);
        }

        //
        let mut guard = self.mux.lock().unwrap();
        *guard = true;
        let _gurad = self.condvar.wait_while(guard, |x| *x);

        if self.closed.load(Ordering::Acquire)
        {
            return None;
        }

        let v = self.queue.pop().unwrap();
        Some(v)
    }

    pub fn close(&self) {
        let mut guard = self.mux.lock().unwrap();
        *guard = false;
        self.closed.store(true, Ordering::Release);
        self.condvar.notify_one();
    }
}