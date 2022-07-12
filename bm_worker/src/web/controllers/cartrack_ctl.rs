use actix_web::web;

use log::{debug, error};
use serde::{Deserialize, Serialize};


use cffc_base::model::returndata::{self, ReturnDataType};
use cffc_base::util::utils;

use crate::web::{AppState, proto};

use crate::dao::model::{CfCartrack, CfCoi};
use std::sync::Arc;
use crate::app_ctx::AppCtx;
use crate::error::{AppResult, AppError};
use crate::web::svc::cartrack_svc;
use crate::web::proto::cartrack::CartrackBo;

/*
pageSize, _ := web_util.GetInt64Value(s.GetString("pageSize"))
pageNo, _ := web_util.GetInt64Value(s.GetString("pageNo"))

camera := util.CleanString(s.GetString("camera"))
startTime := util.CleanString(s.GetString("startTime"))
endTime := util.CleanString(s.GetString("endTime"))
plateContent := util.CleanString(s.GetString("plateContent"))

plateType := util.CleanString(s.GetString("plateType"))
carType := util.CleanString(s.GetString("carType"))
carColor := util.CleanString(s.GetString("carColor"))
 */


//----------------- list -------------------------------
#[derive(Serialize, Deserialize, Debug)]
pub struct ListResult {
    pub page: proto::DataPage,
    pub list: Vec<CartrackBo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListFormData {
    #[serde(rename = "pageSize")]
    pub page_size: Option<String>,

    #[serde(rename = "pageNo")]
    pub page_no: Option<String>,

    pub camera: Option<String>,

    #[serde(rename = "startTime")]
    pub start_time: Option<String>,

    #[serde(rename = "endTime")]
    pub end_time: Option<String>,

    pub alarm: Option<String>,

    #[serde(rename = "plateContent")]
    pub plate_content: Option<String>,

    #[serde(rename = "plateType")]
    pub plate_type: Option<String>,

    #[serde(rename = "carType")]
    pub car_type: Option<String>,

    #[serde(rename = "carColor")]
    pub car_color: Option<String>,

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
    if utils::option_must_notempty(&form.start_time) || utils::option_must_notempty(&form.end_time) {
        // 验证时间字符串
        let valid = utils::option_must_datetime(&form.start_time, utils::DATETIME_FMT_SHORT)
            && utils::option_must_datetime(&form.end_time, utils::DATETIME_FMT_SHORT);
        if !valid {
            return Err("invalid startTime / endTime".to_string());
        }
    }

    if !utils::option_should_num_range(&form.alarm, -1, 1) {
        return Err("invalid alarm".to_string());
    }

    Ok(())
}

pub async fn list(app_state: web::Data<AppState>, form: web::Query<ListFormData>) -> ReturnDataType<ListResult> {
    debug!("cartrack_ctl, form: {:?}", form);

    if let Err(e) = check_list_param(&form) {
        return returndata::fail(format!("{}", e).as_str());
    }

    let page_size = utils::get_option_must_num(&form.page_size);
    let page_no = utils::get_option_must_num(&form.page_no);

    let camera = utils::clean_option_string(&form.camera);
    let alarm = utils::get_option_num(&form.alarm);
    let start_time = utils::clean_option_string(&form.start_time);
    let end_time = utils::clean_option_string(&form.end_time);
    let plate_content = utils::clean_space_option_string(&form.plate_content);
    let plate_type = utils::clean_option_string(&form.plate_type);
    let car_type = utils::clean_option_string(&form.car_type);
    let car_color = utils::clean_option_string(&form.car_color);
    let date_range = utils::DateRange::from_option_str(&start_time, &end_time, utils::DATETIME_FMT_SHORT);

    // 查询摄像头列表
    let ctx = app_state.ctx.clone();
    let camera_list = web::block(move || {
        ctx.web_dao.get_all_sourcelist()
    }).await;
    if let Err(e) = camera_list {
        error!("error, cartrack_ctl, get_all_sourcelist, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let camera_list = camera_list.unwrap();

    // 查询 coi_group 列表
    let ctx = app_state.ctx.clone();
    let group_list = web::block(move || {
        ctx.web_dao.get_coigroup_list()
    }).await;
    if let Err(e) = group_list {
        error!("error, cartrack_ctl, get_coigroup_list, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let group_list = group_list.unwrap();

    // 查询总数
    let ctx = app_state.ctx.clone();
    let camera_cl = camera.clone();
    let date_range_cl = date_range.clone();
    let plate_content_cl = plate_content.clone();
    let plate_type_cl = plate_type.clone();
    let car_type_cl = car_type.clone();
    let car_color_cl = car_color.clone();
    let alarm_cl = alarm.clone();

    let total = web::block(move || {
        ctx.web_dao.get_cartrack_total(camera_cl, date_range_cl,
                                       alarm_cl, plate_content_cl, plate_type_cl,
                                       car_type_cl, car_color_cl)
    }).await;

    if let Err(e) = total {
        error!("error, cartrack_ctl, get_cartrack_total, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let total = total.unwrap();
    if total.is_none() {
        error!("error, cartrack_ctl, get_cartrack_total return null");
        return returndata::fail("can't get total");
    }
    let total = total.unwrap();
    debug!("cartrack_ctl, total:{}", total);

    // 查询分页数据
    let dp = proto::DataPage::new(total as u64,
                                  page_size as u64, page_no as u64);


    let ctx = app_state.ctx.clone();
    let camera_cl = camera.clone();
    let date_range_cl = date_range.clone();
    let plate_content_cl = plate_content.clone();
    let plate_type_cl = plate_type.clone();
    let car_type_cl = car_type.clone();
    let car_color_cl = car_color.clone();
    let start_index = dp.get_start_index();
    let alarm_cl = alarm.clone();

    let cartrack_list = web::block(move || {
        ctx.web_dao.get_cartrack_datapage(camera_cl, date_range_cl, alarm_cl,
                                          plate_content_cl, plate_type_cl,
                                          car_type_cl, car_color_cl,
                                          page_size, start_index as i64)
    }).await;
    if let Err(e) = cartrack_list {
        error!("error, cartrack_ctl, get_cartrack_datapage, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let cartrack_list = cartrack_list.unwrap();
    debug!("cartrack_ctl, cartrack_list:{}", cartrack_list.len());

    let mut bo_list = Vec::new();
    for po in cartrack_list.iter() {
        let ctx = app_state.ctx.clone();
        let coi_match = get_match_coi(ctx, po).await;
        if let Err(e) = coi_match {
            error!("error, cartrack_ctl, get_match_coi, {:?}", e);
            return returndata::fail(format!("{:?}", e).as_str());
        }
        let coi_match = coi_match.unwrap();
        let bo = cartrack_svc::to_bo(po, &group_list, &camera_list,
                                     &app_state.ctx.cfg.dfimg_url, coi_match);
        bo_list.push(bo);
    }

    returndata::success(ListResult {
        page: dp,
        list: bo_list,
    })
}

///
async fn get_match_coi(ctx: Arc<AppCtx>, track: &CfCartrack) -> AppResult<Option<CfCoi>> {
    let judege = track.plate_judged == 1;
    if !judege {
        return Ok(None);
    }

    let plate_content = utils::clean_option_string(&track.plate_content);
    let plate_content = match plate_content {
        Some(v) => v,
        None => {
            return Ok(None);
        }
    };

    let po = web::block(move || {
        ctx.web_dao.load_cfcoi_by_plate(&plate_content)
    }).await;

    if let Err(e) = po {
        return Err(AppError::new(format!("{:?}", e).as_str()));
    }

    let po = po.unwrap();
    Ok(po)
}