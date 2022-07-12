pub mod create_person;
pub mod detect;
pub mod save_db;
pub mod stage_stat;
pub mod signal_proc;

use std::sync::Arc;

use log::{debug};
use tokio::sync::broadcast::Receiver;
use tokio::task::JoinHandle as TkJoinHandle;

use crate::cfg::AppCtx;

pub trait Service {
    fn run(self, rx: Receiver<i64>) -> TkJoinHandle<()>;
}

pub struct ServiceRepo {
    pub ctx: Arc<AppCtx>,
    pub handlers: Vec<TkJoinHandle<()>>,
}


impl ServiceRepo {
    pub fn new(ctx: Arc<AppCtx>) -> Self {
        ServiceRepo {
            ctx,
            handlers: Vec::new(),
        }
    }

    pub fn start_service(&mut self, s: impl Service) {
        let rx = self.ctx.exit_tx.subscribe();
        self.handlers.push(s.run(rx));
    }

    pub async fn join(self) {
        for h in self.handlers {
            let _ = h.await;
            debug!("repo  handle joined.")
        }
    }
}

//----------------------------