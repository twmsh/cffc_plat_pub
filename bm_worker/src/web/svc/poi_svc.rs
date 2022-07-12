use log::error;

use cffc_base::model::img_file;

use crate::dao::model::{CfDfdb, CfPoi};

use crate::web::proto::poi::{PoiBo, PoiBoFace};

fn find_group(sid: &str, db_list: &Vec<CfDfdb>) -> Option<CfDfdb> {
    db_list.iter().find_map(|x| {
        if x.db_sid.eq_ignore_ascii_case(sid) {
            Some(x.clone())
        } else {
            None
        }
    })
}

fn to_boface_list(po: &CfPoi, url_prefix: &str) -> Vec<PoiBoFace> {
    let items = img_file::get_item_from_idscores(po.feature_ids.as_str());
    if let Err(e) = items {
        error!("error, get_item_from_idscores:{}, {:?}", po.feature_ids, e);
        return vec![];
    }
    let items = items.unwrap();
    items.iter().map(|&(id, score)| {
        PoiBoFace {
            face_id: id,
            img_url: img_file::get_person_img_url(url_prefix, po.poi_sid.as_str(), id),
            score,
        }
    }).collect()
}


pub fn to_bo(po: &CfPoi, db_list: &Vec<CfDfdb>, url_prefix: &str) -> PoiBo {
    let cover_url = match po.cover {
        Some(1) => Some(img_file::get_person_cover_url(url_prefix, po.poi_sid.as_str())),
        _ => None
    };
    let faces = to_boface_list(po, url_prefix);
    PoiBo {
        sid: po.poi_sid.clone(),
        cover: po.cover.map_or(0, |x| x as i64),
        cover_url,
        faces,
        detail: po.clone(),
        group: find_group(po.db_sid.as_str(), db_list),
    }
}

pub fn to_bo_list(po_list: &Vec<CfPoi>, db_list: &Vec<CfDfdb>, url_prefix: &str) -> Vec<PoiBo> {
    let mut list = Vec::new();
    for po in po_list.iter() {
        list.push(to_bo(po, db_list, url_prefix));
    }
    list
}



