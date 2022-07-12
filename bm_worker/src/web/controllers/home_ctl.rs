use cffc_base::model::returndata::{self, ReturnDataType};
use actix_web::web;
use crate::web::AppState;
use crate::web::proto::camera::{CameraItem};
use crate::web::svc::camera_svc;
use log::{debug, error};
use crate::queue_item::{QI, FtQI, CtQI};


pub async fn get_display_cameras(app_state: web::Data<AppState>) -> ReturnDataType<Vec<CameraItem>> {

    // 加载source list
    let ctx = app_state.ctx.clone();
    let src_list = web::block(move || {
        ctx.web_dao.get_sourcelist_for_display(4)
    }).await;

    if let Err(e) = src_list {
        error!("error, home_ctl, get_sourcelist_for_display, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let src_list = src_list.unwrap();

    // 加载 dfnode list
    let ctx = app_state.ctx.clone();
    let node_list = web::block(move || {
        ctx.web_dao.get_dfnode_list()
    }).await;
    if let Err(e) = node_list {
        error!("error, home_ctl, get_dfnode_list, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let node_list = node_list.unwrap();

    let item_list = camera_svc::to_cameraitem_list(&src_list, &node_list,
                                                   app_state.ctx.cfg.local_ip.as_str(), app_state.ctx.cfg.live_port as i64);

    if let Err(e) = item_list {
        error!("error, home_ctl, to_cameraitem_list, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }


    returndata::success(item_list.unwrap())
}

pub async fn get_init_alarm_list(app_state: web::Data<AppState>)
                                 -> ReturnDataType<Vec<QI>> {
    let limit = 12;
    let prefix = &app_state.ctx.cfg.dfimg_url;


    let ctx = app_state.ctx.clone();
    let camera_list = web::block(move || {
        ctx.web_dao.get_all_sourcelist()
    }).await;
    if let Err(e) = camera_list {
        error!("error, home_ctl, get_all_sourcelist, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let camera_list = camera_list.unwrap();

    let ctx = app_state.ctx.clone();
    let db_list = web::block(move || {
        ctx.web_dao.get_dfdb_list()
    }).await;
    if let Err(e) = db_list {
        error!("error, home_ctl, get_dfdb_list, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let db_list = db_list.unwrap();

    let ctx = app_state.ctx.clone();
    let car_group_list = web::block(move || {
        ctx.web_dao.get_coigroup_list()
    }).await;
    if let Err(e) = car_group_list {
        error!("error, home_ctl, get_dfdb_list, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let car_group_list = car_group_list.unwrap();


    let ctx = app_state.ctx.clone();
    let facetrack_list = web::block(move || {
        ctx.web_dao.load_latest_facetrack_alarm_list(limit)
    }).await;
    if let Err(e) = facetrack_list {
        error!("error, home_ctl, load_latest_facetrack_alarm_list, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let facetrack_list = facetrack_list.unwrap();


    let ctx = app_state.ctx.clone();
    let cartrack_list = web::block(move || {
        ctx.web_dao.load_latest_cartrack_alarm_list(limit)
    }).await;
    if let Err(e) = cartrack_list {
        error!("error, home_ctl, load_latest_cartrack_alarm_list, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let cartrack_list = cartrack_list.unwrap();


    let mut qi_list = Vec::new();

    //人脸记录
    for v in facetrack_list.iter() {
        let camera = camera_list.iter().find_map(|x| {
            if x.src_sid.eq(&v.src_sid) {
                Some(x)
            } else {
                None
            }
        });

        let match_poi = match v.most_person {
            Some(ref sid) => {
                let ctx = app_state.ctx.clone();
                let poi_sid = sid.clone();
                let po = web::block(move || {
                    ctx.dao.load_poi_by_sid(&poi_sid)
                }).await;
                if let Err(e) = po {
                    error!("error, home_ctl, load_poi_by_sid, {:?}", e);
                    return returndata::fail(format!("{:?}", e).as_str());
                }
                po.unwrap()
            }
            None => None,
        };

        let qi_ft = FtQI::from_po(prefix, v, camera, &db_list, match_poi);
        if let Err(e) = qi_ft {
            error!("error, home_ctl, FtQI::from_po, {:?}", e);
            return returndata::fail(format!("{:?}", e).as_str());
        }
        let qi_ft = qi_ft.unwrap();
        qi_list.push(QI::FT(qi_ft));
    }

    // 车辆记录
    for v in cartrack_list.iter() {
        let camera = camera_list.iter().find_map(|x| {
            if x.src_sid.eq(&v.src_sid) {
                Some(x)
            } else {
                None
            }
        });

        let match_coi = match v.plate_content {
            Some(ref plate) => {
                let ctx = app_state.ctx.clone();
                let coi_plate = plate.clone();
                let po = web::block(move || {
                    ctx.dao.load_coi_by_plate(&coi_plate)
                }).await;
                if let Err(e) = po {
                    error!("error, home_ctl, load_coi_by_plate, {:?}", e);
                    return returndata::fail(format!("{:?}", e).as_str());
                }
                po.unwrap()
            }
            None => None,
        };

        let qi_ct = CtQI::from_po(prefix, v, camera, &car_group_list, match_coi);
        if let Err(e) = qi_ct {
            error!("error, home_ctl, CtQI::from_po, {:?}", e);
            return returndata::fail(format!("{:?}", e).as_str());
        }
        let qi_ct = qi_ct.unwrap();

        qi_list.push(QI::CT(Box::new(qi_ct)));
    }

    // 排序
    qi_list.sort_by_key(|x| {
        match x {
            QI::FT(v) => v.face.ts,
            QI::CT(v) => v.car.ts,
        }
    });

    // 去掉多余的
    if qi_list.len() > limit as usize {
        let remove = qi_list.len() - limit as usize;
        qi_list.drain(0..remove);
    }

    // 倒序
    qi_list.reverse();

    debug!("home_ctl, load: {} alarm tracks", qi_list.len());
    returndata::success(qi_list)
}