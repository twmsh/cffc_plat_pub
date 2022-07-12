#![allow(clippy::too_many_arguments)]

use std::fs::File;

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use bm_worker::dao::model::CfDfsource;
use cffc_base::api::bm_api;
use cffc_base::db::dbop::DbOp;

use crate::dao::AppDao;
use crate::error::{AppError, AppResult};

/// imp_src
// 1) 根据配置文件，调用api添加摄像头
// 2）在本地Sqlite添加cf_dfsource记录
// 3) 支持清空摄像头 -x
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateConfig {
    analysis_node: String,
    notify_url: String,
    face_src: i64,
    car_src: i64,
    mix_src: i64,
    face_urls: Vec<String>,
    car_urls: Vec<String>,
    mix_urls: Vec<String>,
    config: bm_api::CreateSourceReqConfig,
}

impl CreateConfig {
    pub fn load(path: &str) -> AppResult<CreateConfig> {
        let f = File::open(path)?;
        let cfg: CreateConfig = serde_json::from_reader(f)?;
        Ok(cfg)
    }
}


pub struct ImpSrcCmd {
    pub db_url: String,

    /// 采集端api
    pub ana_api: bm_api::AnalysisApi,

    /// config json文件路径
    pub config: String,

    pub dao: AppDao,
    pub remove: bool,
}

impl ImpSrcCmd {
    pub fn new(db_url: &str, a_url: &str, config: &str, remove: bool) -> Self {
        let a_api = bm_api::AnalysisApi::new(a_url);
        let conn = rusqlite::Connection::open(db_url).unwrap();
        let dao = AppDao::new(conn);

        ImpSrcCmd {
            db_url: db_url.to_string(),
            ana_api: a_api,
            config: config.to_string(),
            remove,
            dao,
        }
    }

    pub async fn clean_db_sources(&self) -> AppResult<()> {
        let tables = vec!["cf_dfsource"];

        for table in tables {
            match self.dao.delete_table(table) {
                Ok(v) => {
                    println!("del table: {}({}), ok.", table, v);
                }
                Err(e) => {
                    println!("del table: {}, fail, err:{:?}", table, e);
                }
            };
        }

        Ok(())
    }
    pub async fn clean_api_sources(&self) -> AppResult<()> {
        // 查找所有摄像头，逐个删除
        let res = self.ana_api.get_sources().await?;
        if res.code != 0 {
            return Err(AppError::new(format!("get_sources, return code:{}, msg:{}", res.code, res.msg).as_str()));
        }

        if res.sources.is_none() {
            println!("has no sources, skip it.");
            return Ok(());
        }

        let sources = res.sources.unwrap();
        if sources.is_empty() {
            println!("has no sources, skip it.");
            return Ok(());
        }

        println!("find {} sources, do delete ...", sources.len());

        for (i, v) in sources.iter().enumerate() {
            let del_res = self.ana_api.delete_source(v.id.clone()).await;
            let mut deleted = false;
            let mut err_msg = String::new();
            if let Err(e) = del_res {
                err_msg = format!("{:?}", e);
            } else {
                let del_res = del_res.unwrap();
                if del_res.code == 0 {
                    deleted = true;
                } else {
                    err_msg = format!("delete_source:{}, return code:{}, msg:{}", v.id, del_res.code, del_res.msg);
                }
            }

            if deleted {
                println!("{}, delete source:{}, ok.", i, v.id.clone());
            } else {
                println!("{}, delete source:{}, fail, err: {}", i, v.id.clone(), err_msg);
            }
        }

        Ok(())
    }

    pub async fn create_one(&mut self, src_num: i32, src_type: i64, sid: &str, url: &str,
                            notify_url: &str,
                            src_cfg: &bm_api::CreateSourceReqConfig, node_sid: &str) -> AppResult<()> {
        let name: String;
        let mut src_cfg = src_cfg.clone();
        let now = Local::now();

        match src_type {
            1 => {
                //人脸
                name = format!("{}_face", src_num);
                src_cfg.enable_face = true;
                src_cfg.enable_vehicle = false;
            }
            2 => {
                //车辆
                name = format!("{}_car", src_num);
                src_cfg.enable_face = false;
                src_cfg.enable_vehicle = true;
            }
            3 => {
                //人脸+车辆
                name = format!("{}_mix", src_num);
                src_cfg.enable_face = true;
                src_cfg.enable_vehicle = true;
            }
            _ => {
                return Err(AppError::new(format!("unknown src_type:{}", src_type).as_str()));
            }
        }

        // api创建
        let config_content = serde_json::to_string(&src_cfg)?;
        let res = self.ana_api.create_source(Some(sid.to_string()),
                                             url.to_string(), notify_url.to_string(), src_cfg).await?;

        if res.code != 0 {
            return Err(AppError::new(format!("create_source return code:{}, msg:{}", res.code, res.msg).as_str()));
        }


        // sqlite保存
        let po = CfDfsource {
            id: 0,
            src_sid: sid.to_string(),
            name,
            node_sid: node_sid.to_string(),
            src_url: url.to_string(),
            push_url: notify_url.to_string(),
            ip: "localhost".to_string(),
            src_state: 1,
            src_config: config_content,
            grab_type: src_type as i32,
            io_flag: 0,
            direction: 0,
            tp_id: None,
            upload_flag: 0,
            location_name: None,
            resolution_ratio: None,
            coordinate: None,
            sort_num: 1,
            trip_line: 0,
            rtcp_utc: 0,
            lane_desc: None,
            lane_count: 0,
            memo: None,
            gmt_create: now,
            gmt_modified: now,
        };

        let mut conn = self.dao.conn.lock().unwrap();
        let _id = po.insert(&mut conn)?;

        Ok(())
    }

    pub async fn create_sources(&mut self) -> AppResult<()> {
        let cfg = CreateConfig::load(self.config.as_str())?;
        let mut src_num = 1;
        let mut url_index = 0;

        // 人脸
        for _i in 0..cfg.face_src {
            if cfg.face_urls.is_empty() {
                break;
            }
            let src_url = cfg.face_urls.get(url_index).unwrap();
            let sid = Uuid::new_v4().to_string();
            match self.create_one(src_num, 1, sid.as_str(), src_url.as_str(),
                                  cfg.notify_url.as_str(), &cfg.config,
                                  cfg.analysis_node.as_str()).await {
                Ok(_) => {
                    println!("{}, ok, {}, {}", src_num, sid, src_url);
                }
                Err(e) => {
                    println!("{}, fail, error: {:?}", src_num, e);
                }
            };

            src_num += 1;
            url_index += 1;
            if url_index == cfg.face_urls.len() {
                url_index = 0;
            }
        }

        // 车辆
        url_index = 0;
        for _i in 0..cfg.car_src {
            if cfg.car_urls.is_empty() {
                break;
            }
            let src_url = cfg.car_urls.get(url_index).unwrap();
            let sid = Uuid::new_v4().to_string();
            match self.create_one(src_num, 2, sid.as_str(),
                                  src_url.as_str(), cfg.notify_url.as_str(),
                                  &cfg.config, cfg.analysis_node.as_str()).await {
                Ok(_) => {
                    println!("{}, ok, {}, {}", src_num, sid, src_url);
                }
                Err(e) => {
                    println!("{}, fail, error: {:?}", src_num, e);
                }
            };

            src_num += 1;
            url_index += 1;
            if url_index == cfg.car_urls.len() {
                url_index = 0;
            }
        }

        // 人脸+车辆
        url_index = 0;
        for _i in 0..cfg.mix_src {
            if cfg.mix_urls.is_empty() {
                break;
            }
            let src_url = cfg.mix_urls.get(url_index).unwrap();
            let sid = Uuid::new_v4().to_string();
            match self.create_one(src_num, 3, sid.as_str(), src_url.as_str(),
                                  cfg.notify_url.as_str(), &cfg.config
                                  , cfg.analysis_node.as_str()).await {
                Ok(_) => {
                    println!("{}, ok, {}, {}", src_num, sid, src_url);
                }
                Err(e) => {
                    println!("{}, fail, error: {:?}", src_num, e);
                }
            };

            src_num += 1;
            url_index += 1;
            if url_index == cfg.mix_urls.len() {
                url_index = 0;
            }
        }
        Ok(())
    }


    pub async fn run_cmd(&mut self) -> AppResult<()> {
        // 删除 source (api和sqlite)
        if self.remove {
            self.clean_api_sources().await?;
            return self.clean_db_sources().await;
        }

        self.create_sources().await
    }
}