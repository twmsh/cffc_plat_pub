use cffc_base::model::img_file;

use crate::dao::model::{CfDfdb, CfDfsource, CfFacetrack, CfPoi};
use crate::web::proto::facetrack::{FacetrackBo, FtBoFace};
use crate::web::svc::poi_svc;

pub fn to_bo(po: &CfFacetrack, db_list: &Vec<CfDfdb>,
             camera_list: &Vec<CfDfsource>, url_prefix: &str, poi: Option<CfPoi>) -> FacetrackBo {
    let mut faces = Vec::new();
    let ids = img_file::get_item_from_idscores(po.img_ids.as_str());

    if let Ok(ids) = ids {
        ids.iter().for_each(|x| {
            let img_url = img_file::get_facetrack_largeimg_url(url_prefix, &po.ft_sid, x.0);
            faces.push(FtBoFace {
                face_id: x.0,
                img_url,
                score: x.1,
            });
        });
    }


    let bg_url = img_file::get_facetrack_bgimg_url(url_prefix, &po.ft_sid);
    let camera = camera_list.iter().find_map(|x| {
        if x.src_sid.eq_ignore_ascii_case(&po.src_sid) {
            Some(x.clone())
        } else {
            None
        }
    });

    let match_poi = match poi {
        Some(v) => Some(poi_svc::to_bo(&v, db_list, url_prefix)),
        None => None,
    };

    FacetrackBo {
        sid: po.ft_sid.clone(),
        bg_url,
        faces,
        detail: po.clone(),
        camera,
        match_poi,
    }
}

