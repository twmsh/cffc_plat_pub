use std::fs::File;
use std::sync::Arc;

use clap::{App, Arg};
use deadqueue::unlimited::Queue;
use log::{debug, info};
use tokio::sync::watch;

use bm_worker::app_cfg::AppCfg;
use bm_worker::app_ctx::AppCtx;
use bm_worker::error::AppResult;
use bm_worker::services::{car::car_notify::CarNotifyProcSvc,
                          face::face_notify::FaceNotifyProcSvc,
                          ServiceRepo,
                          signal_proc::SignalProcSvc,
                          track_clean::TrackCleanSvc,
};
use bm_worker::services::car::car_judge::CarJudgeSvc;
use bm_worker::services::ent_bus::EntBusSvc;
use bm_worker::services::face::face_judge::FaceJudgeSvc;
use bm_worker::web::server::WebServer;
use cffc_base::api::bm_api::{self, CreateSourceReqConfig};
use cffc_base::util::{self, logger, utils};

// use actix_service::ServiceFactory;

const APP_NAME: &str = "bm_worker";
const APP_VER_NUM: &str = "0.1.0";
// const APP_BUILD: &str = "2020-04-18 14:56:01";

/**
) 命令行参数处理 v
) 读取配置文件 v
) 初始化日志 v
）初始化web+service
) 初始化出错退出
) 中断退出
*/
#[tokio::main]
async fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("RUST_LOG", "actix_web=debug");

    let cli_app = App::new(APP_NAME)
        .version(APP_VER_NUM)
        .about("worker and web for face/car app")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .takes_value(true)
            .value_name("CONFIG")
            .help("config file")
            .default_value("cfg.json")
        )
        .arg(Arg::with_name("camera")
            .short("s")
            .long("camera")
            .takes_value(true)
            .value_name("Camera CONFIG")
            .help("camera config")
            .default_value("camera.json")
        );

    let cli_matches = cli_app.get_matches();
    let config_file = cli_matches.value_of("config").unwrap();
    let camera_file = cli_matches.value_of("camera").unwrap();

    println!("read config: {}", config_file);

    let local_ips = util::get_local_ips();
    let local_ip = local_ips.get(0).map(|x| x.as_str());

    // 读取配置文件
    let mut cfg = AppCfg::load(config_file).unwrap();
    cfg.set_local_ip(local_ip);
    cfg.replace_var();
    println!("cfg: {:?}", cfg);

    // 读取摄像头默认配置, 覆盖DEFAULT_CREATE_SOURCE_REQ_CONFIG
    //  注意释放lock
    let camera_cfg = load_src_cfg(camera_file).unwrap();
    {
        let mut default_camera_cfg = bm_api::DEFAULT_CREATE_SOURCE_REQ_CONFIG.write().unwrap();
        default_camera_cfg.cfg = camera_cfg;
    }
    {
        println!("static cfg: {:?}", bm_api::DEFAULT_CREATE_SOURCE_REQ_CONFIG.read().unwrap().cfg);
    }


    // 初始化日志
    let _ = logger::init_app_logger_str(&cfg.log.file, "bm_worker", &cfg.log.level, &cfg.log.lib_level);
    debug!("{:?}", cfg);

    info!("bm_worker start ...");

    // 准备相关目录
    let _ = prepare_dirs(&cfg).await.unwrap();

    // 初始化 app各个模块
    let sql_conn = rusqlite::Connection::open(&cfg.db.url).unwrap();

    let (tx, rx) = watch::channel(1_i64);

    let app_ctx = Arc::new(AppCtx::new(cfg, sql_conn, rx));

    let face_queue = Arc::new(Queue::new());
    let face_judge_queue = Arc::new(Queue::new());

    let car_queue = Arc::new(Queue::new());
    let car_judge_queue = Arc::new(Queue::new());
    let general_queue = Arc::new(Queue::new());

    let face_notify_proc_svc = FaceNotifyProcSvc::new(app_ctx.clone(), face_queue.clone(), face_judge_queue.clone());
    let face_judge_svc = FaceJudgeSvc::new(app_ctx.clone(), face_judge_queue, general_queue.clone());
    let car_notify_proc_svc = CarNotifyProcSvc::new(app_ctx.clone(), car_queue.clone(), car_judge_queue.clone());
    let car_judge_svc = CarJudgeSvc::new(app_ctx.clone(), car_judge_queue, general_queue.clone());

    let ent_bus_svc = EntBusSvc::new(app_ctx.clone(), general_queue.clone());
    let signal_proc_svc = SignalProcSvc::new(tx);
    let ws_queue = ent_bus_svc.get_queue("ws");
    let web_server = WebServer::new(app_ctx.clone(), face_queue, car_queue, ws_queue);


    // 启动各个模块
    let mut svc_repo = ServiceRepo::new(app_ctx.clone());
    svc_repo.start_service(signal_proc_svc);
    svc_repo.start_service(face_notify_proc_svc);
    svc_repo.start_service(face_judge_svc);
    svc_repo.start_service(car_notify_proc_svc);
    svc_repo.start_service(car_judge_svc);
    svc_repo.start_service(ent_bus_svc);

    info!("start web_server");
    svc_repo.start_service(web_server);

    if app_ctx.cfg.disk_clean.enable {
        let track_clean_svc = TrackCleanSvc::new(app_ctx.clone());
        svc_repo.start_service(track_clean_svc);
    }

    svc_repo.join().await;
    info!("app exit.");
}

pub fn load_src_cfg(path: &str) -> AppResult<CreateSourceReqConfig> {
    let f = File::open(path)?;
    let cfg = serde_json::from_reader(f)?;
    Ok(cfg)
}

async fn prepare_dirs(cfg: &AppCfg) -> AppResult<()> {
    let _ = utils::prepare_dir(&cfg.web.upload_path).await?;
    Ok(())
}