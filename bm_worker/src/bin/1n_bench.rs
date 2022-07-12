use std::sync::Arc;

use chrono::prelude::*;
use clap::{App, Arg};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::prelude::*;

use cffc_base::api::bm_api;
use cffc_base::util::logger;

#[derive(Serialize, Deserialize, Debug)]
struct Cfg {
    threads: i64,
    count: i64,
    url: String,
    top: i64,
    threshold: i64,
    db: String,
    data: Vec<SearchData>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SearchData {
    file: String,
    uuid: String,

    #[serde(skip)]
    file_buf: String,

    fea: String,
}

async fn read_file_base64(f: &str) -> std::io::Result<String> {
    let mut content = vec![];
    let mut file = match File::open(f).await {
        Ok(v) => v,
        Err(e) => {
            return Err(e);
        }
    };

    let _ = match file.read_to_end(&mut content).await {
        Ok(v) => v,
        Err(e) => { return Err(e); }
    };

    Ok(base64::encode(content))
}

/*
async fn do_search(api: &bm_api::RecognitionApi, cfg: &Cfg, index: usize) -> std::result::Result<i64, String> {
    let start_time = Local::now();

    let db = cfg.db.clone();
    let top = cfg.top;
    let threshold = cfg.threshold;
    let input = cfg.data.get(index).unwrap();

    // detect
    let res = match api.detect(input.file_buf.clone(), true, false).await {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("detect, {:?}", e));
        }
    };

    if res.code != 0 {
        return Err(format!("detect, code:{}, msg: {}", res.code, res.msg));
    }

    let faces = match res.faces {
        Some(v) => v,
        None => {
            return Err(format!("can't find face in {:?}", input.file));
        }
    };

    let face = match faces.get(0) {
        Some(v) => v,
        None => {
            return Err(format!("can't find face in {:?}", input.file));
        }
    };

    let feature = match face.feature {
        Some(ref v) => v,
        None => {
            return Err(format!("return no feature, {:?}", input.file));
        }
    };

    info!("{}, fea: {}", input.file, feature);

    //search
    let fea_qua = bm_api::ApiFeatureQuality {
        feature: feature.clone(),
        quality: face.score,
    };

    // info!("-->{}", feature);

    let feas = vec![fea_qua];
    let res = match api.search(vec![db], vec![top], vec![threshold], vec![feas]).await {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("search, {:?}", e));
        }
    };
    if res.code != 0 {
        return Err(format!("search, code:{}, msg: {}", res.code, res.msg));
    }

    let persons = match res.persons {
        Some(v) => v,
        None => {
            return Err(format!("return no match, {}", input.file));
        }
    };

    let person = match persons.get(0) {
        Some(v) => {
            v.get(0)
        }
        None => {
            return Err(format!("return no match, {}", input.file));
        }
    };

    let match_person = match person {
        Some(v) => v,
        None => {
            return Err(format!("return no match, {}", input.file));
        }
    };

    if !match_person.id.eq_ignore_ascii_case(&input.uuid) {
        return Err(format!("not same, {}, want:{}, get:{}, score:{}", input.file, input.uuid, match_person.id, match_person.score));
    }

    debug!("search {}, return: {}, score: {}", input.file, match_person.id, match_person.score);


    let end_time = Local::now();
    let dur = end_time.signed_duration_since(start_time).num_milliseconds();
    Ok(dur)
}
*/

/// 图片数据为对齐过的
///
async fn do_search_align(api: &bm_api::RecognitionApi, cfg: &Cfg, index: usize) -> std::result::Result<(i64, i64), String> {
    let start_time = Local::now();

    let db = cfg.db.clone();
    let top = cfg.top;
    let threshold = cfg.threshold;
    let input = cfg.data.get(index).unwrap();

    // detect
    let res = match api.get_features(vec![input.file_buf.clone()], false).await {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("get_features, {:?}", e));
        }
    };
    let middle_time = Local::now();

    if res.code != 0 {
        return Err(format!("get_features, code:{}, msg: {}", res.code, res.msg));
    }

    let faces = match res.features {
        Some(v) => v,
        None => {
            return Err(format!("can't find features in {:?}", input.file));
        }
    };

    let feature = match faces.get(0) {
        Some(v) => v,
        None => {
            return Err(format!("can't find feature in {:?}", input.file));
        }
    };

    // info!("{}, fea: {}",input.file,feature);
    if !feature.eq(input.fea.as_str()) {
        error!("{}, feature isn't equal.", input.file);
    }

    //search
    let fea_qua = bm_api::ApiFeatureQuality {
        feature: feature.clone(),
        quality: 1.0_f64,
    };

    let feas = vec![fea_qua];
    let res = match api.search(vec![db], vec![top], vec![threshold], vec![feas]).await {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("search, {:?}", e));
        }
    };
    if res.code != 0 {
        return Err(format!("search, code:{}, msg: {}", res.code, res.msg));
    }

    let persons = match res.persons {
        Some(v) => v,
        None => {
            return Err(format!("return no match, {}", input.file));
        }
    };

    let person = match persons.get(0) {
        Some(v) => {
            v.get(0)
        }
        None => {
            return Err(format!("return no match, {}", input.file));
        }
    };

    let match_person = match person {
        Some(v) => v,
        None => {
            return Err(format!("return no match, {}", input.file));
        }
    };

    if !match_person.id.eq_ignore_ascii_case(&input.uuid) {
        return Err(format!("not same, {}, want:{}, get:{}, score:{}", input.file, input.uuid, match_person.id, match_person.score));
    }

    debug!("search {}, return: {}, score: {}", input.file, match_person.id, match_person.score);

    let end_time = Local::now();
    let dur_1 = middle_time.signed_duration_since(start_time).num_milliseconds();
    let dur_2 = end_time.signed_duration_since(middle_time).num_milliseconds();

    Ok((dur_1, dur_2))
}

#[tokio::main(core_threads = 32)]
// #[tokio::main]
async fn main() {
    let _ = logger::init_console_logger_str("info");

    let cli_app = App::new("1n_bench")
        .version("0.1.0")
        .about("1:N search")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .takes_value(true)
            .required(true)
            .value_name("config file")
            .help("set config")
            .default_value("cfg.json")
        );

    let cli_matches = cli_app.get_matches();

    let config_fn = cli_matches.value_of("config").unwrap();
    info!("config: {}", config_fn);

    // 读取json数据文件
    info!("loading data ...");

    let mut file = match File::open(config_fn).await {
        Ok(f) => f,
        Err(_) => {
            error!("error, can't open file: {}", config_fn);
            return;
        }
    };

    let mut content = vec![];
    let read_size = match file.read_to_end(&mut content).await {
        Ok(v) => v,
        Err(e) => {
            error!("error, {:?}", e);
            return;
        }
    };

    debug!("read file: {} bytes", read_size);
    let mut config: Cfg = match serde_json::from_slice(content.as_slice()) {
        Ok(v) => v,
        Err(e) => {
            error!("error, parse json, {:?}", e);
            return;
        }
    };


    info!("config: {:?}", config);

    // 读取图片数据
    for v in config.data.iter_mut() {
        let content = match read_file_base64(v.file.as_str()).await {
            Ok(v) => v,
            Err(e) => {
                error!("error, read {}, {:?}", v.file, e);
                return;
            }
        };
        v.file_buf = content;
    }


    let config = Arc::new(config);


    let mut handles = vec![];

    let threads = config.threads;


    // 开始计时
    let start_time = Local::now();

    for i in 0..threads {
        let cfg = config.clone();
        let thread_id = i;

        let h = tokio::spawn(async move {
            let api = bm_api::RecognitionApi::new(cfg.url.as_str());
            let count = cfg.count;
            let input_len = cfg.data.len();
            let mut input_index = 0_usize;

            let mut cc = 0_i64;
            let mut succ = 0_i64;
            let mut total_use_1 = 0_i64;
            let mut total_use_2 = 0_i64;


            loop {
                // let rst = do_search(&api, &cfg, input_index).await;
                let rst = do_search_align(&api, &cfg, input_index).await;
                match rst {
                    Ok(v) => {
                        succ += 1;
                        total_use_1 += v.0;
                        total_use_2 += v.1;
                    }
                    Err(e) => {
                        error!("[{}] error, {}", thread_id, e);
                    }
                }

                cc += 1;
                if cc == count {
                    break;
                }

                input_index += 1;
                if input_index == input_len {
                    input_index = 0;
                }

                // thread::sleep(Duration::from_millis(5));
            }
            if succ != 0 {
                let response_time = (total_use_1 + total_use_2) as f64 / succ as f64;

                let response_time_1 = total_use_1 as f64 / succ as f64;
                let response_time_2 = total_use_2 as f64 / succ as f64;

                info!("[{}] succ: {}/{}, average time:{}, {}, {}", thread_id, succ, count,
                      response_time, response_time_1, response_time_2);
            } else {
                info!("[{}] succ: {}/{}", thread_id, succ, count);
            }
            (succ, total_use_1, total_use_2)
        });
        handles.push(h);
    }

    let mut total_succ = 0_i64;
    let mut total_use = 0_i64;

    let mut total_use_1 = 0_i64;
    let mut total_use_2 = 0_i64;

    let mut average_use = 0_f64;

    let mut average_use_1 = 0_f64;
    let mut average_use_2 = 0_f64;

    for h in handles {
        match h.await {
            Ok((succ, use_time_1, use_time_2)) => {
                total_succ += succ;
                total_use += use_time_1 + use_time_2;

                total_use_1 += use_time_1;
                total_use_2 += use_time_2;
            }
            Err(e) => {
                error!("join error, {:?}", e);
            }
        }
    }
    let end_time = Local::now();
    let dur = end_time.signed_duration_since(start_time).num_milliseconds();

    let total_task = threads * config.count;
    let tps: f64 = total_task as f64 / (dur as f64 / 1000_f64);
    if total_succ != 0 {
        average_use = total_use as f64 / total_succ as f64;
        average_use_1 = total_use_1 as f64 / total_succ as f64;
        average_use_2 = total_use_2 as f64 / total_succ as f64;
    }


    info!("");
    info!("{} threads, tasks: {}/{}, duration: {} mills, tps: {}", threads, total_succ, total_task, dur, tps);
    info!("average succ response: {} mills, {}, {}", average_use, average_use_1, average_use_2);

    info!("exit.");
}