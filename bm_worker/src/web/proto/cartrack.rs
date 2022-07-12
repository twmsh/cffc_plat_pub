use serde::{Serialize, Deserialize};
use crate::dao::model::{CfCartrack, CfDfsource};
use crate::web::proto::coi::CoiBo;

#[derive(Serialize, Deserialize, Debug)]
pub struct CtBoCar {
    pub id: i64,
    pub img_url: String,
    pub score: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CtBoPlate {
    pub content: String,
    pub img_url: String,
    pub score: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CtBoProps {
    /// 车身颜色
    pub color: String,

    /// 品牌
    pub brand: String,

    /// 车系
    pub top_series: String,

    /// 车款
    pub series: String,

    /// 车粗分类别
    pub top_type: String,

    /// 车类别
    pub mid_type: String,

    /// 车辆方向
    pub direct: String,

    /// 运动方向，0 未知；1 向上；2 向下
    pub move_direct: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CartrackBo {
    pub sid: String,
    pub bg_url: String,

    pub cars: Vec<CtBoCar>,
    pub plate: Option<CtBoPlate>,
    pub props: Option<CtBoProps>,

    pub detail: CfCartrack,
    pub camera: Option<CfDfsource>,

    #[serde(rename = "match")]
    pub match_coi: Option<CoiBo>,
}