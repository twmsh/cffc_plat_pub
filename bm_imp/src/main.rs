use std::sync::Arc;

use clap::{App, Arg};
use deadqueue::unlimited::Queue;
use log::{debug, error, info};
use regex::Regex;
use tokio::fs;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use bm_imp::cfg::{self, AppCfg, AppCtx, ImpPersonInfo, StageEvent};
use bm_imp::dir_filter::DirWalkFilter;
use bm_imp::error::AppResult;
use cffc_base::util::logger;

use bm_imp::services::{create_person::CreatePersonService,
                       detect::DetectService,
                       save_db::SaveDbService,
                       ServiceRepo,
                       // signal_proc::SignalProcSvc,
                       stage_stat::StageStatService,
};

//----------------------------------------------------------
async fn read_cfg(path: &str) -> AppResult<AppCfg> {
    let content = fs::read_to_string(path).await?;
    let cfg: AppCfg = serde_json::from_reader(content.as_bytes())?;

    Ok(cfg)
}

fn print_dir_filter(df: &DirWalkFilter, props: &[String]) {
    info!("[good sample]:");
    for v in df.good_samples.iter() {
        let info = ImpPersonInfo::from_filename(v, &df.regex, props).unwrap();
        info!("{:?}", info);
    }

    info!("[bad sample]:");
    for v in df.bad_samples.iter() {
        info!("{:?}", v);
    }
}


/// clap处理命令行参数
/// 解析json，获取cfg对象
/// 初始日志
/// 扫描目录，获取文件列表
/// 检测是否是test，如果是，打印信息，退出
/// [file_queue]->(detect)->[fea_queue]->(create)->[person_queue]->(db save)->[end]
#[tokio::main]
async fn main() {
    let cli_app = App::new("bm_imp")
        .version("0.1.0")
        .about("bm imp tools")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .required(true)
            .default_value("cfg.json")
            .value_name("json cfg")
            .help("set cfg file"))
        .arg(Arg::with_name("debug")
            .short("d")
            .long("debug")
            .help("set debug log"))
        .arg(Arg::with_name("exe")
            .short("x")
            .long("exe")
            .help("do imp, ignore test")
        );

    let cli_matches = cli_app.get_matches();
    let present_debug = cli_matches.is_present("debug");
    let present_x = cli_matches.is_present("exe");

    let cfg_fn = cli_matches.value_of("config").unwrap();

    // 读取配置文件
    println!("load config: {}", cfg_fn);
    let mut app_cfg = read_cfg(cfg_fn).await.unwrap();
    println!("{:?}", app_cfg);

    if present_debug {
        app_cfg.log.level = "debug".to_string();
        app_cfg.log.lib_level = "debug".to_string();
    }

    if present_x {
        app_cfg.imp.test = false;
    }

    // 初始化日志
    match logger::init_app_logger_str(app_cfg.log.file.as_str(), "bm_imp", app_cfg.log.level.as_str(), app_cfg.log.lib_level.as_str()) {
        Ok(_) => {}
        Err(_) => {
            println!("error, init_app_logger_str fail.");
            return;
        }
    }
    debug!("init logger, ok.");

    // 文件名称正则
    let reg = match Regex::new(app_cfg.imp.pattern.as_str()) {
        Ok(v) => v,
        Err(e) => {
            error!("{}", e);
            error!("error, invalid pattern: {}", app_cfg.imp.pattern);
            return;
        }
    };

    let capture_size = reg.captures_len() - 1;
    if capture_size != app_cfg.imp.props.len() {
        error!("regex group size:{} not equal props size:{}", capture_size, app_cfg.imp.props.len());
        return;
    }
    if !cfg::check_props(&app_cfg.imp.props) {
        error!("invalid props: {:?}", app_cfg.imp.props);
        return;
    }


    // 枚举目录文件
    let mut dir_filter = DirWalkFilter::new(app_cfg.imp.img_dir.as_str(),
                                            app_cfg.imp.file_ext.clone(), reg.clone());
    info!("list dir: {}", app_cfg.imp.img_dir);

    let _ = dir_filter.list_dir().await.unwrap();
    info!("find files: {}, ok: {}", dir_filter.count, dir_filter.targets.len());

    if app_cfg.imp.test {
        print_dir_filter(&dir_filter, &app_cfg.imp.props);
        info!("test mode, don't import, will exit.");
        return;
    }

    // 初始化appctx
    let sql_conn = rusqlite::Connection::open(&app_cfg.db_url).unwrap();
    let (exit_tx, _) = broadcast::channel(10);
    let (stat_tx, stat_rx) = mpsc::channel::<StageEvent>(10);
    // let exit_tx2 = exit_tx.clone();

    let app_ctx = Arc::new(AppCtx::new(app_cfg, sql_conn, exit_tx, stat_tx));

    // 创建Services
    let mut svc_repo = ServiceRepo::new(app_ctx.clone());
    let fea_queue = Arc::new(Queue::new());
    let person_queue = Arc::new(Queue::new());

    // 启动 stat service
    let files_count = dir_filter.targets.len();
    let stage_service = StageStatService::new(app_ctx.clone(), files_count, stat_rx);
    svc_repo.start_service(stage_service);


    // 启动多个detect service
    let input_files = Arc::new(dir_filter.targets);
    let detect_num = app_ctx.cfg.detect_worker as usize;
    let mut url_index = 0_usize;
    for i in 0..detect_num {
        // helper url 轮换
        let api_url = app_ctx.cfg.recog.helper.get(url_index).unwrap();
        let detect_service = DetectService::new(app_ctx.clone(), input_files.clone(),
                                                api_url, fea_queue.clone(), i);
        svc_repo.start_service(detect_service);

        url_index += 1;
        if url_index == app_ctx.cfg.recog.helper.len() {
            url_index = 0;
        }
    }
    info!("start {} detect serivces", detect_num);

    // 启动多个createperson service
    let create_num = app_ctx.cfg.create_worker as usize;
    for i in 0..create_num {
        let create_service = CreatePersonService::new(app_ctx.clone(), fea_queue.clone(), person_queue.clone(), i);
        svc_repo.start_service(create_service);
    }
    info!("start {} create serivces", create_num);


    let save_service = SaveDbService::new(app_ctx.clone(), person_queue.clone(), reg, 0);
    svc_repo.start_service(save_service);
    info!("start {} save serivce", 1);


    /*
    let signal_service = SignalProcSvc::new(exit_tx2);
    svc_repo.start_service(signal_service);
    info!("start signal serivce");
    */

    svc_repo.join().await;
    info!("app exit.");
}
