use std::{fs::File};

use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgVersion {
    pub product: String,
    pub ver: String,
    pub api_ver: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgLog {
    pub file: String,
    pub level: String,
    pub lib_level: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgApi {
    pub grab_url: String,
    pub recg_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgDb {
    pub url: String,
    pub tz: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgRecvMode {
    pub fast: bool,
    pub count: usize,
    pub quality: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgNotifyProcFt {
    pub recv_mode: AppCfgRecvMode,
    pub wl_alarm: bool,
    /// millisecond
    pub clear_delay: u64,
    /// millisecond
    pub ready_delay: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgNotifyProCt {
    pub recv_mode: AppCfgRecvMode,
    pub wl_alarm: bool,
    /// millisecond
    pub clear_delay: u64,
    /// millisecond
    pub ready_delay: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgNotifyProc {
    /// 不做匹配
    pub skip_search: bool,

    pub debug: bool,
    /// 比对worker的数量
    pub search_worker: u64,
    /// 每次search参与的数量
    pub search_batch: u64,
    pub facetrack: AppCfgNotifyProcFt,
    pub cartrack: AppCfgNotifyProCt,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgWs {
    pub batch: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgWebNode {
    pub sid: String,
    pub url: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgWeb {
    pub notify_url: String,
    pub client_node: AppCfgWebNode,
    pub server_node: AppCfgWebNode,
    pub face_black_db: String,
    pub face_white_db: String,
    pub car_black_group: String,
    pub car_white_group: String,

    pub upload_url: String,
    pub upload_path: String,
    pub use_debug_stream: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgDiskClean {
    pub enable: bool,
    pub avail_size_m: u64,
    pub clean_ft_batch: usize,
    pub clean_ct_batch: usize,
    pub interval_minute: usize,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfg {
    pub version: AppCfgVersion,
    pub log: AppCfgLog,

    pub http_port: u16,
    pub live_port: u16,

    pub df_imgs: String,
    pub dfimg_url: String,
    pub api: AppCfgApi,
    pub db: AppCfgDb,
    pub notify_proc: AppCfgNotifyProc,

    pub ws: AppCfgWs,
    pub web: AppCfgWeb,
    pub disk_clean: AppCfgDiskClean,

    #[serde(default)]
    pub local_ip: String,
}

impl AppCfg {
    pub fn load(path: &str) -> AppResult<AppCfg> {
        let f = File::open(path)?;
        let cfg = serde_json::from_reader(f)?;
        Ok(cfg)
    }

    pub fn set_local_ip(&mut self, ip: Option<&str>) {
        if let Some(v) = ip {
            self.local_ip = v.to_string();
        } else {
            self.local_ip = "127.0.0.1".to_string();
        }
    }

    pub fn validate(&self) -> AppResult<()> {
        Ok(())
    }

    pub fn replace_var(&mut self) {
        let http_port = self.http_port.to_string();

        self.web.notify_url = self.web.notify_url.replace("${http_port}", &http_port);
    }
}



