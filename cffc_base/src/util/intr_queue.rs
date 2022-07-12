use std::collections::vec_deque::VecDeque;
use std::sync::{Condvar, Mutex};

// use futures::SinkExt;

const QUEQE_CAP: usize = 64;

// spsc 模式

pub struct InnerObj<T> {
    queue: VecDeque<T>,
    exited: bool,
    blocked: bool,
}

pub struct IntrQueue<T> {
    pub mux: Mutex<InnerObj<T>>,
    pub condvar: Condvar,
}

impl<T> Default for IntrQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}


impl<T> IntrQueue<T> {
    pub fn new() -> IntrQueue<T> {
        IntrQueue {
            mux: Mutex::new(InnerObj {
                queue: VecDeque::with_capacity(QUEQE_CAP),
                exited: false,
                blocked: true,
            }),
            condvar: Condvar::new(),
        }
    }

    pub fn put(&self, v: T) -> bool {
        let mut guard = self.mux.lock().unwrap();
        if guard.exited {
            // 有退出标志位
            return false;
        }
        guard.queue.push_back(v);
        guard.blocked = false;
        self.condvar.notify_one();
        true
    }

    pub fn get(&self) -> Option<T> {
        let mut guard = self.mux.lock().unwrap();

        if guard.exited {
            // 有退出标志位
            println!("queue left: {}/{}", guard.queue.len(), guard.queue.capacity());
            return None;
        }

        if !guard.queue.is_empty() {
            return Some(guard.queue.pop_front().unwrap());
        }

        guard.blocked = true;

        // 缩减 vcDeq内存
        if guard.queue.capacity() > QUEQE_CAP {
            // println!("queue shrink: {}/{}", guard.queue.len(), guard.queue.capacity());
            guard.queue.shrink_to_fit();
        }

        let mut _guard = self.condvar
            .wait_while(guard,
                        |pair| !pair.exited && pair.blocked).unwrap();
        if _guard.exited {
            println!("queue left: {}/{}.", _guard.queue.len(), _guard.queue.capacity());
            return None;
        }

        let item = _guard.queue.pop_front().unwrap();
        Some(item)
    }

    pub fn close(&self) {
        let mut guard = self.mux.lock().unwrap();
        guard.exited = true;
        self.condvar.notify_one();
    }
}
