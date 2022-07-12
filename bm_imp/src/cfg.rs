use std::ffi::{OsStr, OsString};
use std::path::Path;

use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::{Sender as BSender};
use tokio::sync::mpsc::{Sender as MSender};


use crate::error::{AppError, AppResult};
use std::sync::Mutex;
use std::cell::RefCell;

//-----------------
pub struct AppDao {
    pub conn: RefCell<rusqlite::Connection>,
}

//-----------------
pub struct AppCtx {
    pub cfg: AppCfg,
    pub dao: Mutex<AppDao>,
    pub exit_tx: BSender<i64>,

    pub stat_tx: MSender<StageEvent>,
}

impl AppCtx {
    pub fn new(cfg: AppCfg, conn: rusqlite::Connection,
               exit_tx: BSender<i64>,
               stat_tx: MSender<StageEvent>) -> Self {
        AppCtx {
            cfg,
            dao: Mutex::new(AppDao {
                conn: RefCell::new(conn),
            }),
            exit_tx,
            stat_tx,
        }
    }
}


//---------------
#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgLog {
    pub file: String,
    pub level: String,
    pub lib_level: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgRecog {
    pub url: String,
    pub db_sid: String,
    pub helper: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgImp {
    pub img_dir: String,
    pub file_ext: Vec<String>,
    pub imp_tag: String,
    pub sex_fromid: bool,
    pub test: bool,
    pub threshold: u64,
    pub pattern: String,
    pub props: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfg {
    pub log: AppCfgLog,
    pub recog: AppCfgRecog,
    pub imp: AppCfgImp,
    pub db_url: String,
    pub df_imgs: String,
    pub detect_worker: u64,
    pub create_worker: u64,
    pub create_batch: u64,
    pub save_batch: u64,
}

// ------------------------------
const IMP_PROPS: [&str; 4] = ["name", "sex", "idcard", "memo"];

/// 检查 list中的 props名称是否是以上4个
pub fn check_props(list: &[String]) -> bool {
    for i in list.iter() {
        let mut find = false;
        for j in IMP_PROPS.iter() {
            if j.eq_ignore_ascii_case(i) {
                find = true;
                break;
            }
        }
        if !find {
            return false;
        }
    }
    true
}

pub fn get_gender(sex: &str) -> i8 {
    match sex {
        "男" => 1,
        "女" => 2,
        _ => 0,
    }
}

#[derive(Debug)]
pub struct ImpPersonInfo {
    pub index: u32,
    pub face_id: i64,
    pub score: f64,
    pub person_id: String,

    pub threshold: i32,
    pub imp_tag: Option<String>,

    pub name: String,
    pub gender: i8,
    pub identity_card: String,
    pub memo: String,
}

impl ImpPersonInfo {
    pub fn from_filename(file_name: &OsStr, reg: &Regex, props: &[String]) -> AppResult<Self> {
        let file_stem = Path::new(file_name).file_stem().unwrap().to_str().unwrap();

        let caps = match reg.captures(file_stem) {
            Some(v) => v,
            None => {
                return Err(AppError::COMMON(format!("{} not match regex", file_stem)));
            }
        };

        if caps.len() - 1 != props.len() {
            return Err(AppError::COMMON(format!("{} regex groups not match props", file_stem)));
        }

        let mut info = ImpPersonInfo {
            index: 0,
            face_id: 0,
            score: 1.0,
            person_id: "".to_string(),

            threshold: 0,
            imp_tag: None,

            name: "".to_string(),
            gender: 0,
            identity_card: "".to_string(),
            memo: "".to_string(),
        };

        let count = props.len();
        for i in 0..count {
            let prop = props.get(i).unwrap();
            let value = caps.get(i + 1).unwrap().as_str().to_string();
            match prop.as_str() {
                "name" => {
                    info.name = value;
                }
                "sex" => {
                    info.gender = get_gender(value.as_str());
                }
                "idcard" => {
                    info.identity_card = value;
                }
                "memo" => {
                    info.memo = value;
                }
                _ => {
                    unreachable!("unknown props")
                }
            }
        }

        // name为空时候，用id_card赋值
        if info.name.is_empty() && !info.identity_card.is_empty() {
            info.name = info.identity_card.clone();
        }

        Ok(info)
    }
}

//------------------------------
pub struct TaskStat {
    /// 总数
    pub count: u64,

    /// 操作成功，整个过程调用的累计时长
    pub dur: u64,

    ///成功数量
    pub success: u64,

    /// 操作成功，api调用的累计时长
    pub succ_dur: u64,

}

impl Default for TaskStat {
    fn default() -> Self {
        TaskStat {
            count: 0,
            success: 0,
            dur: 0,
            succ_dur: 0,
        }
    }
}

//------------------------------
#[derive(Debug)]
pub struct FeaItem {
    pub index: u32,
    pub file_name: OsString,
    pub person_id: String,
    pub fea: String,
    pub score: f64,
}

#[derive(Debug)]
pub struct CreateItem {
    pub index: u32,
    pub file_name: OsString,
    pub person_id: String,
    pub face_id: i64,
    pub score: f64,
}


//------------------------------
#[derive(Debug)]
pub struct StageEvent {
    /// 属于那个stage
    pub stage_id: usize,

    /// 该stage可能有多个worker
    pub worker_id: usize,

    /// 处理成功条数
    pub succ: usize,

    /// 处理失败条数
    pub fail: usize,
}

#[derive(Debug)]
pub struct Stage {
    /// 编号
    pub id: usize,

    /// 该stage需要处理的个数，由上一级stage决定
    pub count: usize,

    /// 已经处理的个数，包括成功和失败的
    pub touch: usize,

    /// 处理成功的个数
    pub succ: usize,

    /// 该stage处理完成
    pub done: bool,
}

