use serde::{Deserialize, Serialize};

use crate::dao::model::{CfDfsource, CfFacetrack};
use crate::web::proto::poi::PoiBo;

#[derive(Serialize, Deserialize, Debug)]
pub struct FtBoFace {
    #[serde(rename = "FaceId")]
    pub face_id: i64,

    #[serde(rename = "ImgUrl")]
    pub img_url: String,

    #[serde(rename = "Score")]
    pub score: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FacetrackBo {
    pub sid: String,
    pub bg_url: String,
    pub faces: Vec<FtBoFace>,
    pub detail: CfFacetrack,
    pub camera: Option<CfDfsource>,

    #[serde(rename = "match")]
    pub match_poi: Option<PoiBo>,
}