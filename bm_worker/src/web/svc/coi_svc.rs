use crate::dao::model::{CfCoi, CfCoiGroup};
use crate::web::proto::coi::CoiBo;

fn find_group(sid: &str, db_list: &Vec<CfCoiGroup>) -> Option<CfCoiGroup> {
    db_list.iter().find_map(|x| {
        if x.sid.eq_ignore_ascii_case(sid) {
            Some(x.clone())
        } else {
            None
        }
    })
}


pub fn to_bo(po: &CfCoi, group_list: &Vec<CfCoiGroup>) -> CoiBo {
    CoiBo {
        sid: po.sid.clone(),
        detail: po.clone(),
        group: find_group(po.group_sid.as_str(), group_list),
    }
}

pub fn to_bo_list(po_list: &Vec<CfCoi>, group_list: &Vec<CfCoiGroup>) -> Vec<CoiBo> {
    let mut list = Vec::new();
    for po in po_list.iter() {
        list.push(to_bo(po, group_list));
    }
    list
}

