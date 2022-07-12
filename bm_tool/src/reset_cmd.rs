use std::path::PathBuf;

use tokio::fs;

use cffc_base::api::bm_api;

use crate::dao::AppDao;
use crate::error::{AppError, AppResult};
use std::io::ErrorKind;

/// reset
// 1) 清除7001中的摄像头
// 2）清除7002中的db
// 3) 清除本地sqlite表中的
// cf_dfsource / cf_facetrack / cf_poi / cf_delpoi / cf_coi / cf_cartrack
// 4) 清除本地df_imgs中的 facetrack/ person / cartrack 子目录
pub struct ResetCmd {
    pub db_url: String,

    /// 采集端api
    pub ana_api: bm_api::AnalysisApi,

    /// 识别端api
    pub recg_api: bm_api::RecognitionApi,

    /// 图片目录
    pub img_dir: String,

    pub dao: AppDao,
}

impl ResetCmd {
    pub fn new(db_url: &str, a_url: &str, r_url: &str, img_dir: &str) -> Self {
        let a_api = bm_api::AnalysisApi::new(a_url);
        let r_api = bm_api::RecognitionApi::new(r_url);
        let conn = rusqlite::Connection::open(db_url).unwrap();
        let dao = AppDao::new(conn);

        ResetCmd {
            db_url: db_url.to_string(),
            ana_api: a_api,
            recg_api: r_api,
            img_dir: img_dir.to_string(),
            dao,
        }
    }

    async fn clean_7001(&self) -> AppResult<()> {
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

    async fn clean_7002(&self) -> AppResult<()> {
        // 查找所有db，逐个删除
        let res = self.recg_api.get_dbs().await?;
        if res.code != 0 {
            return Err(AppError::new(format!("get_dbs, return code:{}, msg:{}", res.code, res.msg).as_str()));
        }

        if res.dbs.is_none() {
            println!("has no dbs, skip it.");
            return Ok(());
        }

        let dbs = res.dbs.unwrap();
        if dbs.is_empty() {
            println!("has no dbs, skip it.");
            return Ok(());
        }

        println!("find {} dbs, do delete ...", dbs.len());


        for (i, v) in dbs.iter().enumerate() {
            let del_res = self.recg_api.delete_db(v.clone()).await;
            let mut deleted = false;
            let mut err_msg = String::new();
            if let Err(e) = del_res {
                err_msg = format!("{:?}", e);
            } else {
                let del_res = del_res.unwrap();
                if del_res.code == 0 {
                    deleted = true;
                } else {
                    err_msg = format!("delete_db:{}, return code:{}, msg:{}", v, del_res.code, del_res.msg);
                }
            }

            if deleted {
                println!("{}, delete db:{}, ok.", i, v);
            } else {
                println!("{}, delete db:{}, fail, err: {}", i, v, err_msg);
            }
        }

        Ok(())
    }

    async fn init_7002(&self) -> AppResult<()> {
        // 读取sqlite中的数据，调用api，初始化dbs

        let dbs = self.dao.get_dbs()?;
        if dbs.is_empty() {
            println!("warn, can't find dbs in sqlite");
            return Ok(());
        }
        println!("find {} dbs, will create.", dbs.len());

        for (i, v) in dbs.iter().enumerate() {
            let create_res = self.recg_api.create_db(Some(v.0.clone()), v.1).await;
            let mut created = false;
            let mut err_msg = String::new();

            if let Err(e) = create_res {
                err_msg = format!("{:?}", e);
            } else {
                let create_res = create_res.unwrap();
                if create_res.code == 0 {
                    created = true;
                } else {
                    err_msg = format!("delete_db:({},{}), return code:{}, msg:{}", v.0, v.1,
                                      create_res.code, create_res.msg);
                }
            }
            if created {
                println!("{}, create db:({},{}), ok.", i, v.0, v.1);
            } else {
                println!("{}, create db:({},{}), fail, err: {}", i, v.0, v.1, err_msg);
            }
        }


        Ok(())
    }

    async fn clean_sqlite(&self) -> AppResult<()> {
        // cf_dfsource / cf_facetrack / cf_poi / cf_delpoi / cf_coi / cf_cartrack

        let tables = vec!["cf_dfsource", "cf_facetrack", "cf_poi", "cf_delpoi", "cf_cartrack", "cf_coi"];

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

    /// img_dir 需要存在 ，子目录可以不存在
    async fn clean_img(&self) -> AppResult<()> {
        let sub_dirs = vec!["facetrack", "cartrack", "person"];

        // 检查 img_dir 是否存在
        let meta = fs::metadata(self.img_dir.as_str()).await?;
        if !meta.is_dir() {
            return Err(AppError::new(format!("{} isn't dir", self.img_dir).as_str()));
        }

        for sub_dir in sub_dirs {
            let mut path = PathBuf::from(self.img_dir.clone());
            path.push(sub_dir);

            match fs::remove_dir_all(path.as_os_str()).await {
                Ok(_) => {
                    println!("remove dir:{:?}, ok.", path);
                }
                Err(e) => {
                    match e.kind() {
                        ErrorKind::NotFound => {
                            println!("remove dir:{:?}, not exsit, skip", path);
                        }
                        _ => {
                            println!("remove dir:{:?}, fail, err:{:?}", path, e);
                        }
                    }
                }
            }
        }

        Ok(())
    }


    pub async fn run_cmd(&mut self) -> AppResult<()> {
        println!("ResetCmd, run_cmd");

        if let Err(e) = self.clean_7001().await {
            println!("error, {:?}", e);
        }

        if let Err(e) = self.clean_7002().await {
            println!("error, {:?}", e);
        }

        if let Err(e) = self.init_7002().await {
            println!("error, {:?}", e);
        }

        if let Err(e) = self.clean_sqlite().await {
            println!("error, {:?}", e);
        }

        if let Err(e) = self.clean_img().await {
            println!("error, {:?}", e);
        }

        Ok(())
    }
}
