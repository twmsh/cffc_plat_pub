use std::sync::Arc;

use actix_web::web;
use log::{debug, error};
use serde::{Deserialize, Serialize};

use cffc_base::model::returndata::{self, ReturnDataType};
use cffc_base::util::utils;

use crate::app_ctx::AppCtx;
use crate::dao::model::{CfFacetrack, CfPoi};
use crate::error::{AppError, AppResult};
use crate::web::{AppState, proto};
use crate::web::proto::facetrack::FacetrackBo;

use crate::web::svc::facetrack_svc;

//----------------- list -------------------------------
#[derive(Serialize, Deserialize, Debug)]
pub struct ListResult {
    pub page: proto::DataPage,
    pub list: Vec<FacetrackBo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListFormData {
    #[serde(rename = "pageSize")]
    pub page_size: Option<String>,

    #[serde(rename = "pageNo")]
    pub page_no: Option<String>,

    pub camera: Option<String>,
    pub alarm: Option<String>,
    pub name: Option<String>,

    #[serde(rename = "identityCard")]
    pub identity_card: Option<String>,

    pub gender: Option<String>,

    #[serde(rename = "startTime")]
    pub start_time: Option<String>,

    #[serde(rename = "endTime")]
    pub end_time: Option<String>,
}

fn check_list_param(form: &web::Query<ListFormData>) -> std::result::Result<(), String> {
    // 必填
    if !utils::option_must_length(&form.page_size, 1, 1000) {
        return Err("invalid pageSize".to_string());
    }

    if !utils::option_must_length(&form.page_no, 1, 10000_0000) {
        return Err("invalid pageNo".to_string());
    }

    //选填
    if !utils::option_should_num_range(&form.gender, 1, 2) {
        return Err("invalid gender".to_string());
    }

    if !utils::option_should_num_range(&form.alarm, -1, 1) {
        return Err("invalid alarm".to_string());
    }

    if utils::option_must_notempty(&form.start_time) || utils::option_must_notempty(&form.end_time) {
        // 验证时间字符串
        let valid = utils::option_must_datetime(&form.start_time, utils::DATETIME_FMT_SHORT)
            && utils::option_must_datetime(&form.end_time, utils::DATETIME_FMT_SHORT);
        if !valid {
            return Err("invalid startTime / endTime".to_string());
        }
    }

    Ok(())
}

/*
	pageSize, _ := strconv.ParseInt(CleanString(ctl.GetString("pageSize")), 10, 64)
	pageNo, _ := strconv.ParseInt(CleanString(ctl.GetString("pageNo")), 10, 64)

	camera := CleanString(ctl.GetString("camera"))
	startTime := CleanString(ctl.GetString("startTime"))
	endTime := CleanString(ctl.GetString("endTime"))
	alarm := util.AsNumber(CleanString(ctl.GetString("alarm")), -1)

	name := CleanString(ctl.GetString("name"))
	identityCard := CleanString(ctl.GetString("identityCard"))
	gender := util.AsNumber(CleanString(ctl.GetString("gender")), -1)
 */

pub async fn list(app_state: web::Data<AppState>, form: web::Query<ListFormData>) -> ReturnDataType<ListResult> {
    debug!("facetrack_ctl, form: {:?}", form);

    if let Err(e) = check_list_param(&form) {
        return returndata::fail(format!("{}", e).as_str());
    }

    let page_size = utils::get_option_must_num(&form.page_size);
    let page_no = utils::get_option_must_num(&form.page_no);

    let camera = utils::clean_option_string(&form.camera);
    let start_time = utils::clean_option_string(&form.start_time);
    let end_time = utils::clean_option_string(&form.end_time);
    let alarm = utils::get_option_num(&form.alarm);
    let name = utils::clean_option_string(&form.name);
    let identity_card = utils::clean_option_string(&form.identity_card);
    let gender = utils::get_option_num(&form.gender);
    let date_range = utils::DateRange::from_option_str(&start_time, &end_time, utils::DATETIME_FMT_SHORT);

    // 查询摄像头列表
    let ctx = app_state.ctx.clone();
    let camera_list = web::block(move || {
        ctx.web_dao.get_all_sourcelist()
    }).await;
    if let Err(e) = camera_list {
        error!("error, facetrack_ctl, get_all_sourcelist, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let camera_list = camera_list.unwrap();


    // 查询db列表
    let ctx = app_state.ctx.clone();
    let db_list = web::block(move || {
        ctx.web_dao.get_dfdb_list()
    }).await;
    if let Err(e) = db_list {
        error!("error, facetrack_ctl, get_dfdb_list, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let db_list = db_list.unwrap();


    // 查询总数
    let ctx = app_state.ctx.clone();
    let camera_cl = camera.clone();
    let date_range_cl = date_range.clone();
    let alarm_cl = alarm.clone();
    let name_cl = name.clone();
    let identity_card_cl = identity_card.clone();
    let gender_cl = gender.clone();

    let total = web::block(move || {
        ctx.web_dao.get_facetrack_total(camera_cl, date_range_cl,
                                        alarm_cl, name_cl, identity_card_cl,
                                        gender_cl)
    }).await;
    if let Err(e) = total {
        error!("error, facetrack_ctl, get_facetrack_total, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let total = total.unwrap();
    if total.is_none() {
        error!("error, facetrack_ctl, get_facetrack_total return null");
        return returndata::fail("can't get total");
    }
    let total = total.unwrap();


    // 查询分页数据
    let dp = proto::DataPage::new(total as u64,
                                  page_size as u64, page_no as u64);
    let ctx = app_state.ctx.clone();
    let camera_cl = camera.clone();
    let date_range_cl = date_range.clone();
    let alarm_cl = alarm.clone();
    let name_cl = name.clone();
    let identity_card_cl = identity_card.clone();
    let gender_cl = gender.clone();
    let start_index = dp.get_start_index();

    let facetrack_list = web::block(move || {
        ctx.web_dao.get_facetrack_datapage(camera_cl, date_range_cl,
                                           alarm_cl, name_cl, identity_card_cl,
                                           gender_cl, page_size, start_index as i64)
    }).await;
    if let Err(e) = facetrack_list {
        error!("error, facetrack_ctl, get_facetrack_datapage, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let facetrack_list = facetrack_list.unwrap();

    let mut bo_list = Vec::new();
    for po in facetrack_list.iter() {
        let ctx = app_state.ctx.clone();
        let poi_match = get_match_poi(ctx, po).await;

        if let Err(e) = poi_match {
            error!("error, facetrack_ctl, get_match_poi, {:?}", e);
            return returndata::fail(format!("{:?}", e).as_str());
        }
        let poi_match = poi_match.unwrap();
        let bo = facetrack_svc::to_bo(po, &db_list, &camera_list,
                                      &app_state.ctx.cfg.dfimg_url, poi_match);
        bo_list.push(bo);
    }

    returndata::success(ListResult {
        page: dp,
        list: bo_list,
    })
}

async fn get_match_poi(ctx: Arc<AppCtx>, ft: &CfFacetrack) -> AppResult<Option<CfPoi>> {
    let judged = ft.judged.map_or(false, |x| x == 1);
    if !judged {
        return Ok(None);
    }

    let poi_sid = match ft.most_person {
        Some(ref v) => v.clone(),
        None => {
            return Ok(None);
        }
    };

    let po = web::block(move || {
        ctx.web_dao.load_cfpoi_by_sid(&poi_sid)
    }).await;

    if let Err(e) = po {
        return Err(AppError::new(format!("{:?}", e).as_str()));
    }

    let po = po.unwrap();
    Ok(po)
}