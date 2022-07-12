use signal_hook::{SIGINT, SIGTERM, SIGQUIT, iterator::Signals};

use crate::services::Service;

use tokio::task::{JoinHandle as TkJoinHandle};

use tokio::sync::watch::{Sender, Receiver};

use log::{info};


pub struct SignalProcSvc {
    tx: Sender<i64>,
    signals: Signals,
}

impl SignalProcSvc {
    pub fn new(tx: Sender<i64>) -> Self {
        let signals = Signals::new(&[SIGTERM, SIGINT, SIGQUIT]).unwrap();

        SignalProcSvc {
            tx,
            signals,
        }
    }
}

impl Service for SignalProcSvc {
    fn run(self, _rx: Receiver<i64>) -> TkJoinHandle<()> {
        tokio::task::spawn_blocking(move || {
            self.signals.wait();
            info!("SignalProcSvc catch exit event, broadcast it");
            let _ = self.tx.broadcast(100);
            info!("SignalProcSvc exit.");
        })
    }
}