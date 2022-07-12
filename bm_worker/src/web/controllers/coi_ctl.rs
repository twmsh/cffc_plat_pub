use actix_web::web;
use chrono::prelude::*;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use cffc_base::model::returndata::{self, ReturnDataType};
use cffc_base::util::utils;

use crate::dao::model::{CfCoi, CfCoiGroup};
use crate::web::AppState;
use crate::web::proto;
use crate::web::proto::coi::CoiBo;
use crate::web::svc::coi_svc;

pub async fn group_list(app_state: web::Data<AppState>) -> ReturnDataType<Vec<CfCoiGroup>> {
    let ctx = app_state.ctx.clone();
    let list = web::block(move || {
        ctx.web_dao.get_coigroup_list()
    }).await;

    if let Err(e) = list {
        error!("error, coi_ctl, get_coigroup_list, {:?}", e);
        return returndata::fail("query db fail");
    }
    let list = list.unwrap();
    returndata::success(list)
}

//----------------- list -------------------------------
#[derive(Serialize, Deserialize, Debug)]
pub struct ListResult {
    pub page: proto::DataPage,
    pub list: Vec<CoiBo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListFormData {
    #[serde(rename = "pageSize")]
    pub page_size: Option<String>,

    #[serde(rename = "pageNo")]
    pub page_no: Option<String>,

    pub name: Option<String>,
    pub plate: Option<String>,
    pub phone: Option<String>,
    pub group: Option<String>,
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

    Ok(())
}

pub async fn list(app_state: web::Data<AppState>,
                  form: web::Query<ListFormData>) -> ReturnDataType<ListResult> {
    if let Err(e) = check_list_param(&form) {
        return returndata::fail(format!("{}", e).as_str());
    }

    let page_size = utils::get_option_must_num(&form.page_size);
    let page_no = utils::get_option_must_num(&form.page_no);
    let name = utils::clean_option_string(&form.name);
    let plate = utils::clean_option_string(&form.plate);
    let phone = utils::clean_option_string(&form.phone);
    let group = utils::clean_option_string(&form.group);

    // 查询 coi_group 列表
    let ctx = app_state.ctx.clone();
    let group_list = web::block(move || {
        ctx.web_dao.get_coigroup_list()
    }).await;
    if let Err(e) = group_list {
        error!("error, coi_ctl, get_coigroup_list, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let group_list = group_list.unwrap();

    // 查询总数
    let ctx = app_state.ctx.clone();
    let name_cl = name.clone();
    let plate_cl = plate.clone();
    let phone_cl = phone.clone();
    let group_cl = group.clone();

    let total = web::block(move || {
        ctx.web_dao.get_coi_total(name_cl, plate_cl, phone_cl,
                                  group_cl)
    }).await;
    if let Err(e) = total {
        error!("error, poi_ctl, get_coi_total, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let total = total.unwrap();
    if total.is_none() {
        error!("error, get_coi_total, get_coi_total return null");
        return returndata::fail("can't get total");
    }
    let total = total.unwrap();
    debug!("coi_ctl, get_coi_total: {}", total);


    // 查询分页数据
    let dp = proto::DataPage::new(total as u64,
                                  page_size as u64, page_no as u64);

    let ctx = app_state.ctx.clone();
    let name_cl = name.clone();
    let plate_cl = plate.clone();
    let phone_cl = phone.clone();
    let group_cl = group.clone();
    let start_index = dp.get_start_index();

    let coi_list = web::block(move || {
        ctx.web_dao.get_coi_datapage(name_cl, plate_cl, phone_cl,
                                     group_cl,
                                     page_size, start_index as i64)
    }).await;
    if let Err(e) = coi_list {
        error!("error, coi_ctl, get_coi_datapage, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let coi_list = coi_list.unwrap();
    debug!("coi_ctl, coi_list:{}", coi_list.len());

    let bo_list = coi_svc::to_bo_list(&coi_list, &group_list);

    returndata::success(ListResult {
        page: dp,
        list: bo_list,
    })
}


//----------------- detail -------------------------------
#[derive(Serialize, Deserialize, Debug)]
pub struct DetailFormData {
    pub sid: Option<String>,
}

fn check_detail_param(form: &web::Query<DetailFormData>) -> std::result::Result<(), String> {
    if !utils::option_must_length(&form.sid, 1, 50) {
        return Err("invalid sid".to_string());
    }

    Ok(())
}

pub async fn detail(app_state: web::Data<AppState>,
                    form: web::Query<DetailFormData>) -> ReturnDataType<CoiBo> {
    if let Err(e) = check_detail_param(&form) {
        return returndata::fail(e.as_str());
    }

    let sid = form.sid.as_ref().unwrap();

    // 查找poi
    let ctx = app_state.ctx.clone();
    let po_sid = sid.clone();
    let po = web::block(move || {
        ctx.web_dao.load_cfcoi_by_sid(po_sid.as_str())
    }).await;
    if let Err(e) = po {
        error!("error, coi_ctl, load_cfcoi_by_sid:{}, {:?}", sid, e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let po = po.unwrap();
    if po.is_none() {
        error!("error, coi_ctl, can't find coi:{}", sid);
        return returndata::fail(format!("can't find coi: {}", sid).as_str());
    }
    let po = po.unwrap();

    let ctx = app_state.ctx.clone();
    let group_list = web::block(move || {
        ctx.web_dao.get_coigroup_list()
    }).await;
    if let Err(e) = group_list {
        error!("error, coi_ctl, get_coigroup_list:{}, {:?}", sid, e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let group_list = group_list.unwrap();

    let bo = coi_svc::to_bo(&po, &group_list);
    returndata::success(bo)
}


//----------------- add -------------------------------
#[derive(Serialize, Deserialize, Debug)]
pub struct AddFormData {
    pub group: Option<String>,
    pub plate_content: Option<String>,

    pub plate_type: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub memo: Option<String>,
}

fn check_add_param(form: &web::Form<AddFormData>) -> std::result::Result<(), String> {

    //必填

    if !utils::option_must_length(&form.group, 1, 50) {
        return Err("invalid group".to_string());
    }

    if !utils::option_must_length(&form.plate_content, 1, 50) {
        return Err("invalid plate_content".to_string());
    }


    Ok(())
}

/// 检查参数
/// 检查是否有同名的车牌
/// 保存数据库
pub async fn add(app_state: web::Data<AppState>, form: web::Form<AddFormData>) -> ReturnDataType<String> {
    if let Err(e) = check_add_param(&form) {
        return returndata::fail(e.as_str());
    }

    let group = utils::clean_option_string(&form.group).unwrap();
    let plate_content = utils::clean_space_option_string(&form.plate_content).unwrap();

    let plate_type = utils::clean_option_string(&form.plate_type);
    let name = utils::clean_option_string(&form.name);
    let phone = utils::clean_option_string(&form.phone);
    let memo = utils::clean_option_string(&form.memo);
    let now = Local::now();

    let ctx = app_state.ctx.clone();
    let plate_content_cl = plate_content.clone();
    let po = web::block(move || {
        ctx.web_dao.load_cfcoi_by_plate(&plate_content_cl)
    }).await;

    if let Err(e) = po {
        error!("error, coi_ctl, load_cfcoi_by_plate, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }

    let po = po.unwrap();

    if po.is_some() {
        error!("error, coi_ctl, plate name exsit, {}", plate_content);
        return returndata::fail_msg("车牌号码已存在", "invalid plate_content");
    }

    let coi_sid = Uuid::new_v4().to_string();
    let po = CfCoi {
        id: 0,
        sid: coi_sid.clone(),
        group_sid: group.clone(),
        plate_content,
        plate_type,
        car_brand: None,
        car_series: None,
        car_size: None,
        car_type: None,
        owner_name: name,
        owner_idcard: None,
        owner_phone: phone,
        owner_address: None,
        flag: 0,
        tag: None,
        imp_tag: None,
        memo,
        gmt_create: now,
        gmt_modified: now,
    };

    let ctx = app_state.ctx.clone();
    let coi_id = web::block(move || {
        ctx.web_dao.save_cfcoi_for_add(&po)
    }).await;
    if let Err(e) = coi_id {
        error!("error, coi_ctl, save_cfcoi_for_add, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let coi_id = coi_id.unwrap();
    debug!("coi_ctl, save db, cfcoi:{}, id:{}", coi_sid, coi_id);

    returndata::success_str("succ")
}


//----------------- delete -------------------------------
#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteFormData {
    pub sids: Option<String>,
}

fn check_delete_param(form: &web::Form<DeleteFormData>) -> std::result::Result<(), String> {
    if !utils::option_must_length(&form.sids, 1, 50) {
        return Err("invalid sids".to_string());
    }

    Ok(())
}

/// 先删除底层，再删除数据库, 强制删除
pub async fn delete(app_state: web::Data<AppState>, form: web::Form<DeleteFormData>) -> ReturnDataType<String> {
    if let Err(e) = check_delete_param(&form) {
        return returndata::fail(e.as_str());
    }
    let sid = form.sids.as_ref().unwrap();

    let ctx = app_state.ctx.clone();
    let coi_sid = sid.clone();
    let po = web::block(move || {
        ctx.web_dao.load_cfcoi_by_sid(&coi_sid)
    }).await;
    if let Err(e) = po {
        error!("error, coi_ctl, load_cfcoi_by_sid:{}, {:?}", sid, e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let po = po.unwrap();
    if po.is_none() {
        error!("error, coi_ctl, coi not exsit, {}", sid);
        return returndata::fail("coi not exsit");
    }


    // 数据库删除
    let ctx = app_state.ctx.clone();
    let coi_sid = sid.clone();
    let affect = web::block(move || {
        ctx.web_dao.delete_cfcoi_by_sid(&coi_sid)
    }).await;
    if let Err(e) = affect {
        error!("error, coi_ctl, delete_cfcoi_by_sid, {:?}", e);
    } else {
        let affect = affect.unwrap();
        debug!("poi_ctl, delete_cfcoi_by_sid:{}, affect:{}", sid, affect);
    }

    returndata::success_str("succ")
}


//----------------- modify -------------------------------
#[derive(Serialize, Deserialize, Debug)]
pub struct ModifyFormData {
    pub sid: Option<String>,
    pub group: Option<String>,
    pub plate_content: Option<String>,

    pub plate_type: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub memo: Option<String>,
}

fn check_modify_param(form: &web::Form<ModifyFormData>) -> std::result::Result<(), String> {
    if !utils::option_must_length(&form.sid, 1, 50) {
        return Err("invalid sid".to_string());
    }
    if !utils::option_must_length(&form.group, 1, 50) {
        return Err("invalid group".to_string());
    }

    if !utils::option_must_length(&form.plate_content, 1, 300) {
        return Err("invalid plate_content".to_string());
    }

    Ok(())
}

/// 检查名称不重复
/// 检查摄像头是否存在
/// 更新 CfDfsource, 如果scr_state=1(开启状态) ，则需要更新底层
pub async fn modify(app_state: web::Data<AppState>, form: web::Form<ModifyFormData>) -> ReturnDataType<String> {
    if let Err(e) = check_modify_param(&form) {
        return returndata::fail(e.as_str());
    }

    let sid = utils::clean_option_string(&form.sid).unwrap();
    let group = utils::clean_option_string(&form.group).unwrap();
    let plate_content = utils::clean_space_option_string(&form.plate_content).unwrap();

    let plate_type = utils::clean_option_string(&form.plate_type);
    let name = utils::clean_option_string(&form.name);
    let phone = utils::clean_option_string(&form.phone);
    let memo = utils::clean_option_string(&form.memo);
    let now = Local::now();


    // 根据车牌号码查找
    let ctx = app_state.ctx.clone();
    let plate_content_cl = plate_content.clone();
    let po = web::block(move || {
        ctx.web_dao.load_cfcoi_by_plate(&plate_content_cl)
    }).await;
    if let Err(e) = po {
        error!("error, coi_ctl, load_cfcoi_by_plate, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let po = po.unwrap();
    if let Some(ref v) = po {
        if !v.sid.eq(sid.as_str()) {
            // 存在该车牌号，且不是自己，则表示冲突
            debug!("coi_ctl, duplicate plate_conent, {}", sid);
            return returndata::fail_msg("车牌号重复", "duplicate plate_conent");
        }
    }

    // 根据sid查找
    let ctx = app_state.ctx.clone();
    let sid_cl = sid.clone();
    let po = web::block(move || {
        ctx.web_dao.load_cfcoi_by_sid(&sid_cl)
    }).await;
    if let Err(e) = po {
        error!("error, coi_ctl, load_cfcoi_by_sid, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let po = po.unwrap();
    if po.is_none() {
        debug!("coi_ctl, can't find coi:{}", sid);
        return returndata::fail_msg("车牌号不存在", "plate_content not exsit");
    }
    let mut po = po.unwrap();

    po.plate_content = plate_content;
    po.group_sid = group;
    po.plate_type = plate_type;
    po.owner_name = name;
    po.owner_phone = phone;
    po.memo = memo;
    po.gmt_modified = now;

    let ctx = app_state.ctx.clone();
    let affect = web::block(move || {
        ctx.web_dao.update_cfcoi_for_modify(&po)
    }).await;
    if let Err(e) = affect {
        error!("error, coi_ctl, update_cfcoi_for_modify, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let affect = affect.unwrap();
    if affect != 1 {
        error!("error, coi_ctl, update cfcoi, affect:{}", affect);
        return returndata::fail("update fail");
    }

    returndata::success_str("succ")
}