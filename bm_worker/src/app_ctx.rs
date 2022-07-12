use std::sync::Arc;

use tokio::sync::watch::Receiver;

use cffc_base::api::bm_api::{AnalysisApi, RecognitionApi};
use cffc_base::db::SqliteClient;

use crate::app_cfg::AppCfg;
use crate::dao::AppDao;
use crate::dao::web_dao::WebDao;

pub struct AppCtx {
    pub cfg: AppCfg,
    pub dao: AppDao,
    pub web_dao: WebDao,
    pub exit_rx: Receiver<i64>,

    // add
    pub ana_api: AnalysisApi,
    pub recg_api: RecognitionApi,
}

impl AppCtx {
    pub fn new(cfg: AppCfg, conn: rusqlite::Connection, rx: Receiver<i64>) -> Self {
        let sqlite_client = Arc::new(SqliteClient::new(conn));

        AppCtx {
            dao: AppDao::new(sqlite_client.clone()),
            web_dao: WebDao::new(sqlite_client),
            exit_rx: rx,
            ana_api: AnalysisApi::new(cfg.web.client_node.url.as_str()),
            recg_api: RecognitionApi::new(cfg.web.server_node.url.as_str()),
            cfg,
        }
    }
}

