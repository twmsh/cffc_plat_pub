use chrono::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct BeuserBo {
    pub id: i64,
    pub name: String,
    pub login_name: String,
    pub phone: String,

    pub email: String,
    pub service_flag: i32,
    pub last_login: DateTime<Local>,
    pub memo: String,
    pub gmt_create: DateTime<Local>,
    pub gmt_modified: DateTime<Local>,
}