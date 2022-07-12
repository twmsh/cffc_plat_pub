use serde::{Serialize, Deserialize};
use crate::dao::model::{CfCoi, CfCoiGroup};

#[derive(Serialize, Deserialize, Debug)]
pub struct CoiBo {
    pub sid: String,
    pub detail: CfCoi,
    pub group: Option<CfCoiGroup>,
}
