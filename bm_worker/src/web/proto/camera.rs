use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct DfSrcParam {
    pub url: String,
    pub min_face: i64,

    /// 默认值：0 使用原始尺寸
    pub bg_width: i64,

    /// 默认值：-1
    pub area_top: i64,

    /// 默认值：-1
    pub area_left: i64,

    /// 默认值：-1
    pub area_width: i64,

    /// 默认值：-1
    pub area_height: i64,
}

impl Default for DfSrcParam {
    fn default() -> Self {
        DfSrcParam {
            url: "".to_string(),
            min_face: 20,
            bg_width: 0,
            area_top: -1,
            area_left: -1,
            area_width: -1,
            area_height: -1,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CameraItem {
    // from CfDfsource
    pub id: i64,
    pub src_sid: String,
    pub name: String,
    pub node_sid: String,
    pub src_url: String,
    pub push_url: String,
    pub ip: String,
    pub src_state: i32,
    pub src_config: String,
    pub grab_type: i32,
    pub io_flag: i32,
    pub direction: i32,
    pub tp_id: Option<String>,
    pub upload_flag: i32,
    pub location_name: Option<String>,
    pub resolution_ratio: Option<String>,
    pub coordinate: Option<String>,
    pub sort_num: i32,
    pub trip_line: i64,
    pub rtcp_utc: i32,
    pub lane_desc: Option<String>,
    pub lane_count: i32,
    pub memo: Option<String>,
    pub gmt_create: DateTime<Local>,
    pub gmt_modified: DateTime<Local>,

    // added
    pub params: DfSrcParam,
    pub debug_url: String,
}