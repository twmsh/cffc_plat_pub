use serde::{Serialize, Deserialize};
use crate::dao::model::{CfDfdb, CfPoi};

#[derive(Serialize, Deserialize, Debug)]
pub struct PoiBoFace {
    #[serde(rename = "FaceId")]
    pub face_id: i64,

    #[serde(rename = "ImgUrl")]
    pub img_url: String,

    #[serde(rename = "Score")]
    pub score: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PoiBo {
    pub sid: String,
    pub cover: i64,
    pub cover_url: Option<String>,
    pub faces: Vec<PoiBoFace>,

    pub detail: CfPoi,
    pub group: Option<CfDfdb>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImgPathScore {
    pub path: String,
    pub score: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImgAppendItem {
    pub path: String,
    pub score: f64,
    pub feature: String,
    pub face_id: i64,
}
