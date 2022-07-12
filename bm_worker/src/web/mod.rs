use std::sync::Arc;

use deadqueue::unlimited::Queue;
use tera::Tera;

use crate::app_ctx::AppCtx;
use crate::queue_item::{NotifyCarQueueItem, NotifyFaceQueueItem};

pub mod server;
pub mod router;
pub mod controllers;
pub mod api_auth;
pub mod proto;
pub mod svc;

pub struct AppState {
    pub ctx: Arc<AppCtx>,
    pub face_queue: Arc<Queue<NotifyFaceQueueItem>>,
    pub car_queue: Arc<Queue<NotifyCarQueueItem>>,

    pub tmpl: Tera,
}

impl AppState {
    pub fn new(ctx: Arc<AppCtx>, face_queue: Arc<Queue<NotifyFaceQueueItem>>, car_queue: Arc<Queue<NotifyCarQueueItem>>) -> Self {
        let tera = Tera::new("views/**/*.tpl").unwrap();

        AppState {
            ctx,
            face_queue,
            car_queue,
            tmpl: tera,
        }
    }
}
