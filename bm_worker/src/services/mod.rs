pub mod track_clean;
pub mod car;
pub mod face;
pub mod signal_proc;
pub mod ent_bus;
pub mod ws;

use crate::app_ctx::AppCtx;
use std::sync::Arc;
use std::vec::Vec;
use tokio::task::{JoinHandle as TkJoinHandle};
use tokio::sync::watch::{Receiver};
use log::{info};

pub trait Service {
    fn run(self, rx: Receiver<i64>) -> TkJoinHandle<()>;
}

pub struct ServiceRepo {
    app_ctx: Arc<AppCtx>,
    handlers: Vec<TkJoinHandle<()>>,

}

impl ServiceRepo {
    pub fn new(app_ctx: Arc<AppCtx>) -> Self {
        ServiceRepo {
            app_ctx,
            handlers: Vec::new(),
        }
    }

    pub fn start_service(&mut self, s: impl Service) {
        let rx = self.app_ctx.exit_rx.clone();
        self.handlers.push(s.run(rx));
    }

    pub async fn join(self) {
        for h in self.handlers {
            let _ = h.await;
            info!("repo  handle joined.")
        }
    }
}
