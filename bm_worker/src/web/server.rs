use std::sync::{Arc, mpsc};
use std::thread;

use actix::Actor;
use actix_web::{App, http, HttpServer, rt, web};
use actix_web::dev::{Server, Service};
use actix_web::http::{HeaderName, HeaderValue};
use actix_web::middleware::Logger;
use chrono::prelude::*;
use deadqueue::unlimited::Queue;
use log::{error, info};
use tokio::stream::StreamExt;
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle as TkJoinHandle;

use crate::app_ctx::AppCtx;
use crate::queue_item::{NotifyCarQueueItem, NotifyFaceQueueItem};
use crate::queue_item::QI;
use crate::services::Service as CfService;
use crate::services::ws::agent::WsAgent;
use crate::services::ws::worker::WsWorker;
use crate::web::AppState;

use super::api_auth::ApiAuthFilter;
use super::router;

pub struct WebServer {
    ctx: Arc<AppCtx>,
    face_queue: Arc<Queue<NotifyFaceQueueItem>>,
    car_queue: Arc<Queue<NotifyCarQueueItem>>,
    ws_queue: Arc<Queue<QI>>,
}

impl WebServer {
    pub fn new(ctx: Arc<AppCtx>, face_queue: Arc<Queue<NotifyFaceQueueItem>>, car_queue: Arc<Queue<NotifyCarQueueItem>>,
               ws_queue: Arc<Queue<QI>>) -> Self {
        WebServer {
            ctx,
            face_queue,
            car_queue,
            ws_queue,
        }
    }
}

impl CfService for WebServer {
    fn run(self, rx: Receiver<i64>) -> TkJoinHandle<()> {
        let ctx = self.ctx;
        let face_queue = self.face_queue;
        let car_queue = self.car_queue;
        let ws_queue = self.ws_queue;

        let mut exit_rx = rx;

        // 开一个线程
        let (tx_svr, rx_svr) = mpsc::channel::<Server>();

        let _web_handle = thread::spawn(move || {
            let mut sys = rt::System::new("web_server");

            let addr = format!("0.0.0.0:{}", ctx.cfg.http_port);

            let mut ws_worker = WsWorker::new(ctx.clone(), ws_queue);
            if let Err(e) = ws_worker.load() {
                error!("error, WsWorker load error, {:?}", e);
                panic!("WsWorker load error");
            }

            let ws_worker = ws_worker.start();

            let ws_agent = WsAgent::new(ws_worker).start();

            let state = web::Data::new(AppState::new(ctx, face_queue, car_queue));

            let server = HttpServer::new(move || {
                App::new().app_data(state.clone())
                    .data(ws_agent.clone())
                    .app_data(web::PayloadConfig::new(1024 * 1024 * 10))
                    .configure(router::config)
                    .wrap(Logger::default())
                    .wrap(ApiAuthFilter::new("/api/"))
                    .wrap_fn(|req, srv| {
                        let ts_start = Local::now();

                        let req_path = req.path().to_string();
                        let req_orign = match req.headers().get(http::header::ORIGIN) {
                            Some(v) => v.to_str().map_or("*".to_string(), |x| x.to_string()),
                            None => "*".to_string(),
                        };

                        let fut = srv.call(req);
                        async move {
                            let mut res = fut.await?;
                            let ts_use = Local::now() - ts_start;
                            res.headers_mut().insert(
                                HeaderName::from_static("x-cf-use"),
                                HeaderValue::from(ts_use.num_milliseconds()),
                            );

                            if req_path.starts_with("/api/") || req_path.starts_with("/logon") {
                                res.headers_mut().insert(
                                    http::header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                                    HeaderValue::from_str("true").unwrap(),
                                );
                                res.headers_mut().insert(
                                    http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
                                    HeaderValue::from_str(&req_orign).unwrap(),
                                );
                            }


                            Ok(res)
                        }
                    })
            }).disable_signals().bind(addr).unwrap().run();

            tx_svr.send(server.clone()).unwrap();

            let _ = sys.block_on(server);
            info!("after sys.block_on");
        });

        tokio::spawn(async move {
            let srv = rx_svr.recv().unwrap();
            loop {
                let quit = exit_rx.next().await;
                if let Some(100) = quit {
                    info!("WebServer recv exit ...");
                    break;
                }
            }

            let h = thread::spawn(move || {
                rt::System::new("").block_on(srv.stop(false));
                info!("WebServer, webserver stopped.");
            });
            let _ = h.join();
        })
    }
}