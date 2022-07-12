use cffc_base::model::img_file;

use crate::dao::model::{CfCartrack, CfCoiGroup, CfDfsource, CfCoi};
use crate::web::proto::cartrack::{CartrackBo, CtBoCar, CtBoPlate, CtBoProps};
use crate::web::svc::coi_svc;

pub fn to_bo(po: &CfCartrack, group_list: &Vec<CfCoiGroup>,
             camera_list: &Vec<CfDfsource>, url_prefix: &str, coi: Option<CfCoi>) -> CartrackBo {
    let mut cars = Vec::new();
    let ids = img_file::get_item_from_idscores(po.img_ids.as_str());

    if let Ok(ids) = ids {
        ids.iter().for_each(|x| {
            let img_url = img_file::get_cartrack_img_url(url_prefix, &po.sid, x.0);
            cars.push(CtBoCar {
                id: x.0,
                img_url,
                score: x.1,
            });
        });
    }

    let bg_url = img_file::get_cartrack_bgimg_url(url_prefix, &po.sid);
    let camera = camera_list.iter().find_map(|x| {
        if x.src_sid.eq_ignore_ascii_case(&po.src_sid) {
            Some(x.clone())
        } else {
            None
        }
    });

    let match_coi = match coi {
        Some(v) => Some(coi_svc::to_bo(&v, group_list)),
        None => None,
    };

    let plate = match po.plate_judged {
        1 => {
            let v = CtBoPlate {
                content: po.plate_content.as_ref().map_or_else(|| "".to_string(), |x| x.clone()),
                img_url: img_file::get_cartrack_plate_url(url_prefix, po.sid.as_str()),
                score: po.plate_confidence.map_or(1.0, |x| x),
            };
            Some(v)
        }
        _ => None
    };

    let props = match po.vehicle_judged {
        1 => {
            let v = CtBoProps {
                color: po.car_color.as_ref().map_or_else(|| "".to_string(), |x| x.clone()),
                brand: po.car_brand.as_ref().map_or_else(|| "".to_string(), |x| x.clone()),
                top_series: po.car_top_series.as_ref().map_or_else(|| "".to_string(), |x| x.clone()),
                series: po.car_series.as_ref().map_or_else(|| "".to_string(), |x| x.clone()),
                top_type: po.car_top_type.as_ref().map_or_else(|| "".to_string(), |x| x.clone()),
                mid_type: po.car_mid_type.as_ref().map_or_else(|| "".to_string(), |x| x.clone()),
                direct: po.car_direct.as_ref().map_or_else(|| "".to_string(), |x| x.clone()),
                move_direct: po.move_direct as i64,
            };
            Some(v)
        }
        _ => None
    };


    CartrackBo {
        sid: po.sid.clone(),
        bg_url,
        cars,
        plate,
        props,
        detail: po.clone(),
        camera,
        match_coi,
    }
}