#![allow(clippy::collapsible_if)]

use actix_web::{HttpRequest, web};
use chrono::prelude::*;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use cffc_base::api::bm_api::{self, AnalysisApi, CreateSourceReqConfig};
use cffc_base::model::img_file;
use cffc_base::model::returndata::{self, ReturnDataType};
use cffc_base::util::utils;

use crate::dao::model::{BeUser, CfDfsource};
use crate::error::{AppError, AppResult};
use crate::web::AppState;
use crate::web::proto::camera::CameraItem;
use crate::web::svc::camera_svc;

#[derive(Serialize, Deserialize, Debug)]
pub struct DetailData {
    sid: String,
    name: String,
}

pub async fn detail(req: HttpRequest) -> ReturnDataType<DetailData> {
    let mut sid = String::new();
    let mut name = String::new();

    if let Some(po) = req.extensions_mut().get::<BeUser>() {
        sid = po.password.clone();
        name = po.login_name.clone();
        debug!("--> po: {:?}", po);
    }

    returndata::success(DetailData {
        sid,
        name,
    })
}

//----------------- add -------------------------------
#[derive(Serialize, Deserialize, Debug)]
pub struct AddFormData {
    pub name: Option<String>,
    pub url: Option<String>,

    #[serde(rename = "type")]
    pub c_type: Option<String>,

    pub min_face: Option<String>,
}

fn check_add_param(form: &web::Form<AddFormData>) -> std::result::Result<(), String> {
    if !utils::option_must_length(&form.name, 1, 50) {
        return Err("invalid name".to_string());
    }

    if !utils::option_must_length(&form.url, 1, 300) {
        return Err("invalid url".to_string());
    }

    if !utils::option_must_num_range(&form.min_face, 1, 1000) {
        return Err("invalid min_face".to_string());
    }

    if !utils::option_must_num_range(&form.c_type, 1, 3) {
        return Err("invalid type".to_string());
    }

    Ok(())
}

/// 检查参数
/// 检查是否有同名的摄像头
/// 保存数据库，api增加摄像头
/// api增加失败，则删除数据库记录
pub async fn add(app_state: web::Data<AppState>, form: web::Form<AddFormData>) -> ReturnDataType<String> {
    if let Err(e) = check_add_param(&form) {
        return returndata::fail(e.as_str());
    }

    let name = form.name.as_ref().unwrap();
    let url = form.url.as_ref().unwrap();
    let url = &url.trim().to_string();

    let c_type = utils::get_option_must_num(&form.c_type);
    let min_face = utils::get_option_must_num(&form.min_face);
    let now = Local::now();

    let ctx = app_state.ctx.clone();
    let src_name = name.clone();
    let po = web::block(move || {
        ctx.web_dao.load_dfsource_by_name(src_name.as_str())
    }).await;

    if let Err(e) = po {
        error!("error, camera_ctl, load_dfsource_by_name, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }

    let po = po.unwrap();

    if po.is_some() {
        error!("error, camera_ctl, camera name exsit, {}", name);
        return returndata::fail_msg("摄像头名称已存在", "invalid name");
    }

    let src_sid = Uuid::new_v4().to_string();
    let mut src_req_cfg = {
        // 释放锁
        let lock = bm_api::DEFAULT_CREATE_SOURCE_REQ_CONFIG.read();
        let guard = lock.unwrap();
        guard.cfg.clone()
    };

    src_req_cfg.face.min_width = min_face;
    match c_type {
        1 => {
            src_req_cfg.enable_face = true;
            src_req_cfg.enable_vehicle = false;
        }
        2 => {
            src_req_cfg.enable_face = false;
            src_req_cfg.enable_vehicle = true;
        }
        3 => {
            src_req_cfg.enable_face = true;
            src_req_cfg.enable_vehicle = true;
        }
        _ => {
            unreachable!()
        }
    }

    let config_json = serde_json::to_string(&src_req_cfg);
    if let Err(e) = config_json {
        error!("error, camera_ctl, src_req_cfg to json, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let config_json = config_json.unwrap();

    let po = CfDfsource {
        id: 0,
        src_sid: src_sid.clone(),
        name: name.clone(),
        node_sid: app_state.ctx.cfg.web.client_node.sid.clone(),
        src_url: url.clone(),
        push_url: app_state.ctx.cfg.web.notify_url.clone(),
        ip: img_file::get_ip_from_rtsp(url, "localhost"),
        src_state: 1,
        src_config: config_json,
        grab_type: c_type as i32,
        io_flag: 0,
        direction: 0,
        tp_id: None,
        upload_flag: 0,
        location_name: None,
        resolution_ratio: None,
        coordinate: None,
        sort_num: 1,
        trip_line: 0,
        rtcp_utc: 0,
        lane_desc: None,
        lane_count: 0,
        memo: None,
        gmt_create: now,
        gmt_modified: now,
    };

    let ctx = app_state.ctx.clone();
    let src_id = web::block(move || {
        ctx.web_dao.save_dfsource_for_add(&po)
    }).await;
    if let Err(e) = src_id {
        error!("error, camera_ctl, save_dfsource_for_add, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let src_id = src_id.unwrap();
    debug!("camera_ctl, save db, camera:{}, id:{}", src_sid, src_id);

    // api 创建
    let mut created = false;
    let res = app_state.ctx.ana_api.create_source(
        Some(src_sid.clone()), url.clone(),
        app_state.ctx.cfg.web.notify_url.clone(), src_req_cfg).await;
    if let Err(e) = res {
        error!("error, camera_ctl, create_source: {:?}", e);
    } else {
        let res = res.unwrap();
        if res.code == 0 {
            created = true;
        } else {
            error!("error, camera_ctl, create_source, return code:{}, msg:{}", res.code, res.msg);
        }
    }

    if !created {
        // 删除数据库记录
        debug!("camera_ctl, api create fail, will delete in db: {}", src_sid);
        let ctx = app_state.ctx.clone();
        let sid = src_sid.clone();
        let affect = web::block(move || {
            ctx.web_dao.delete_dfsource_by_sid(&sid)
        }).await;

        if let Err(e) = affect {
            error!("error, camera_ctl, delete_dfsource_by_sid: {:?}", e);
        } else {
            let affect = affect.unwrap();
            debug!("camera_ctl, api create fail, delete {}, affect: {}", src_sid, affect);
        }

        return returndata::fail("create fail");
    }

    returndata::success_str("succ")
}

//----------------- modify -------------------------------
#[derive(Serialize, Deserialize, Debug)]
pub struct ModifyFormData {
    pub sid: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,

    #[serde(rename = "type")]
    pub c_type: Option<String>,

    pub min_face: Option<String>,
}

fn check_modify_param(form: &web::Form<ModifyFormData>) -> std::result::Result<(), String> {
    if !utils::option_must_length(&form.sid, 1, 50) {
        return Err("invalid sid".to_string());
    }
    if !utils::option_must_length(&form.name, 1, 50) {
        return Err("invalid name".to_string());
    }

    if !utils::option_must_length(&form.url, 1, 300) {
        return Err("invalid url".to_string());
    }

    if !utils::option_must_num_range(&form.min_face, 1, 1000) {
        return Err("invalid min_face".to_string());
    }

    if !utils::option_must_num_range(&form.c_type, 1, 3) {
        return Err("invalid type".to_string());
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

    let sid = form.sid.as_ref().unwrap();
    let name = form.name.as_ref().unwrap();
    let url = form.url.as_ref().unwrap();
    let url = &url.trim().to_string();

    let c_type = utils::get_option_must_num(&form.c_type);
    let min_face = utils::get_option_must_num(&form.min_face);

    let now = Local::now();

    // 根据名称查找
    let ctx = app_state.ctx.clone();
    let src_name = name.clone();
    let po = web::block(move || {
        ctx.web_dao.load_dfsource_by_name(src_name.as_str())
    }).await;
    if let Err(e) = po {
        error!("error, camera_ctl, load_dfsource_by_name, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let po = po.unwrap();
    if let Some(ref v) = po {
        if !v.src_sid.eq(sid.as_str()) {
            // 存在该名称，且不是自己，则表示名称冲突
            debug!("camera_ctl, duplicate src name:{}", sid);
            return returndata::fail_msg("名称重复", "duplicate name");
        }
    }

    // 根据sid查找
    let ctx = app_state.ctx.clone();
    let src_sid = sid.clone();
    let po = web::block(move || {
        ctx.web_dao.load_dfsource_by_sid(src_sid.as_str())
    }).await;
    if let Err(e) = po {
        error!("error, camera_ctl, load_dfsource_by_sid, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let po = po.unwrap();
    if po.is_none() {
        debug!("camera_ctl, can't find source:{}", sid);
        return returndata::fail_msg("摄像头不存在", "camera not exsit");
    }
    let mut po = po.unwrap();

    //生成 src_config
    let mut src_req_cfg = {
        // 释放锁
        let lock = bm_api::DEFAULT_CREATE_SOURCE_REQ_CONFIG.read();
        let guard = lock.unwrap();
        guard.cfg.clone()
    };

    src_req_cfg.face.min_width = min_face;
    match c_type {
        1 => {
            src_req_cfg.enable_face = true;
            src_req_cfg.enable_vehicle = false;
        }
        2 => {
            src_req_cfg.enable_face = false;
            src_req_cfg.enable_vehicle = true;
        }
        3 => {
            src_req_cfg.enable_face = true;
            src_req_cfg.enable_vehicle = true;
        }
        _ => {
            unreachable!()
        }
    }

    let config_json = serde_json::to_string(&src_req_cfg);
    if let Err(e) = config_json {
        error!("error, camera_ctl, src_req_cfg to json, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let config_json = config_json.unwrap();

    // 底层开启,则需要更新
    if po.src_state == 1 {
        let res = app_state.ctx.ana_api.update_source(sid.clone(), url.clone(),
                                                      app_state.ctx.cfg.web.notify_url.clone(), src_req_cfg).await;

        if let Err(e) = res {
            error!("error, camera_ctl, api update_source, {:?}", e);
            return returndata::fail(format!("{:?}", e).as_str());
        }
        let res = res.unwrap();
        if res.code != 0 {
            error!("error, camera_ctl, api update_source, return code:{}, msg:{}", res.code, res.msg);
            return returndata::fail("update source fail");
        }
    }

    po.name = name.clone();
    po.src_url = url.clone();
    po.ip = img_file::get_ip_from_rtsp(url, "localhost");
    po.grab_type = c_type as i32;
    po.src_config = config_json;
    po.gmt_modified = now;

    let ctx = app_state.ctx.clone();
    let affect = web::block(move || {
        ctx.web_dao.update_dfsource_for_modify(&po)
    }).await;
    if let Err(e) = affect {
        error!("error, camera_ctl, update_dfsource_for_modify, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let affect = affect.unwrap();
    if affect != 1 {
        error!("error, camera_ctl, update dfsource, affect:{}", affect);
        return returndata::fail("update fail");
    }

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

    // api 删除
    let res = app_state.ctx.ana_api.delete_source(sid.clone()).await;
    if let Err(e) = res {
        error!("error, camera_ctl, delete_source, {:?}", e);
    } else {
        let res = res.unwrap();
        debug!(" camera_ctl, api delete_source:{}, return code:{}, msg:{}", sid, res.code, res.msg);
    }

    let ctx = app_state.ctx.clone();
    let src_sid = sid.clone();
    let affect = web::block(move || {
        ctx.web_dao.delete_dfsource_by_sid(&src_sid)
    }).await;
    if let Err(e) = affect {
        error!("error, camera_ctl, delete_dfsource_by_sid, {:?}", e);
    } else {
        let affect = affect.unwrap();
        debug!("camera_ctl, db delete_source:{}, affect:{}", sid, affect);
    }

    returndata::success_str("succ")
}

/*
pub async fn delete_old(app_state: web::Data<AppState>, form: web::Form<DeleteFormData>) -> ReturnDataType<String> {
    if let Err(e) = check_delete_param(&form) {
        return returndata::fail(e.as_str());
    }

    let sids = form.sids.as_ref().unwrap();
    for (i, v) in sids.iter().enumerate() {

        // api 删除
        let res = app_state.ctx.ana_api.delete_source(v.clone()).await;
        if let Err(e) = res {
            error!("error, camera_ctl, delete_source, {:?}", e);
        } else {
            let res = res.unwrap();
            debug!("{}, camera_ctl, api delete_source:{}, return code:{}, msg:{}", i, v, res.code, res.msg);
        }

        let ctx = app_state.ctx.clone();
        let src_sid = v.clone();
        let affect = web::block(move || {
            ctx.web_dao.delete_dfsource_by_sid(&src_sid)
        }).await;
        if let Err(e) = affect {
            error!("error, camera_ctl, delete_dfsource_by_sid, {:?}", e);
        } else {
            let affect = affect.unwrap();
            debug!("{}, camera_ctl, db delete_source:{}, affect:{}", i, v, affect);
        }
    }


    returndata::success_str("succ")
}
*/

//----------------- list -------------------------------
pub async fn list(app_state: web::Data<AppState>) -> ReturnDataType<Vec<CameraItem>> {

    // 加载source list
    let ctx = app_state.ctx.clone();
    let src_list = web::block(move || {
        ctx.web_dao.get_all_sourcelist()
    }).await;

    if let Err(e) = src_list {
        error!("error, camera_ctl, get_all_sourcelist, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let src_list = src_list.unwrap();

    // 加载 dfnode list
    let ctx = app_state.ctx.clone();
    let node_list = web::block(move || {
        ctx.web_dao.get_dfnode_list()
    }).await;
    if let Err(e) = node_list {
        error!("error, camera_ctl, get_dfnode_list, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let node_list = node_list.unwrap();

    let item_list = camera_svc::to_cameraitem_list(&src_list, &node_list,
                                                   app_state.ctx.cfg.local_ip.as_str(), app_state.ctx.cfg.live_port as i64);

    if let Err(e) = item_list {
        error!("error, camera_ctl, to_cameraitem_list, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }

    returndata::success(item_list.unwrap())
}

//----------------- set_on_screen -------------------------------
#[derive(Serialize, Deserialize, Debug)]
pub struct SetOnScreenResult {
    pub src_sid: String,
    pub screen: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetOnScreenFormData {
    pub sid: Option<String>,
    pub screen: Option<String>,
}

fn check_setonscreen_param(form: &web::Form<SetOnScreenFormData>) -> std::result::Result<(), String> {
    if !utils::option_must_length(&form.sid, 1, 50) {
        return Err("invalid sid".to_string());
    }
    if !utils::option_must_num_range(&form.screen, 0, 1) {
        return Err("invalid screen".to_string());
    }
    Ok(())
}

pub async fn set_on_screen(app_state: web::Data<AppState>,
                           form: web::Form<SetOnScreenFormData>) -> ReturnDataType<SetOnScreenResult> {
    if let Err(e) = check_setonscreen_param(&form) {
        return returndata::fail(e.as_str());
    }

    let sid = form.sid.as_ref().unwrap();
    let screen = form.screen.as_ref().unwrap().parse::<i64>().unwrap();

    // 检测摄像头是否存在
    let ctx = app_state.ctx.clone();
    let src_sid = sid.clone();
    let po = web::block(move || {
        ctx.web_dao.load_dfsource_by_sid(src_sid.as_str())
    }).await;

    if let Err(e) = po {
        error!("error, load_dfsource_by_sid, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let po = po.unwrap();
    if po.is_none() {
        error!("error, camera not exsit, {}", sid);
        return returndata::fail_msg("摄像头不存在", "invalid sid");
    }
    let mut po = po.unwrap();
    po.sort_num = screen as i32;
    po.gmt_modified = Local::now();

    let ctx = app_state.ctx.clone();
    let affect = web::block(move || {
        ctx.web_dao.update_dfsource_for_onscreen(&po)
    }).await;
    if let Err(e) = affect {
        error!("error, update_beuser_for_modify, {:?}", e);
        return returndata::fail_msg("更新失败", "update fail");
    }
    let affect = affect.unwrap();
    if affect != 1 {
        error!("error, update_beuser_for_modify, affect: {}", affect);
        return returndata::fail_msg("更新失败", "update fail");
    }

    returndata::success(SetOnScreenResult {
        src_sid: sid.clone(),
        screen,
    })
}

//----------------- set_state -------------------------------
#[derive(Serialize, Deserialize, Debug)]
struct SetStateResult {
    src_sid: String,
    src_state: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct SetStateFormData {
    pub sid: Option<String>,
    pub state: Option<String>,
}

fn check_setstate_param(form: &web::Form<SetStateFormData>) -> std::result::Result<(), String> {
    if !utils::option_must_length(&form.sid, 1, 50) {
        return Err("invalid sid".to_string());
    }
    if !utils::option_must_num_range(&form.state, 0, 1) {
        return Err("invalid screen".to_string());
    }
    Ok(())
}

/// 读取数据库记录，api读取底层数据
///
/// 数据库状态 	底层状态		指令		数据库状态	底层状态
//
// 关闭  		存在			1(开启)   更改      更新
// 关闭  		不存在		1(开启)   更改      创建
// 开启			存在			1(开启)   无操作     无操作
// 开启			不存在		1(开启)   无操作     创建
//
// 关闭  		存在			0(关闭)   无操作     删除
// 关闭  		不存在		0(关闭)   无操作     无操作
// 开启			存在			0(关闭)   更改      删除
// 开启			不存在		0(关闭)   更改      无操作
///
pub async fn set_state(app_state: web::Data<AppState>,
                       form: web::Form<SetStateFormData>) -> ReturnDataType<SetOnScreenResult> {
    if let Err(e) = check_setstate_param(&form) {
        return returndata::fail(e.as_str());
    }

    let sid = form.sid.as_ref().unwrap();
    let state = form.state.as_ref().unwrap().parse::<i64>().unwrap();
    let mut on_db = false;
    let mut on_api = false;

    let mut will_api_update = false;
    let mut will_api_create = false;
    let mut will_api_delete = false;
    let mut will_db_update = false;

    // 在数据库中，查询摄像头
    let ctx = app_state.ctx.clone();
    let src_sid = sid.clone();
    let po = web::block(move || {
        ctx.web_dao.load_dfsource_by_sid(src_sid.as_str())
    }).await;

    if let Err(e) = po {
        error!("error, load_dfsource_by_sid, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let po = po.unwrap();
    if po.is_none() {
        error!("error, camera not exsit, {}", sid);
        return returndata::fail_msg("摄像头不存在", "invalid sid");
    }
    let mut po = po.unwrap();
    if po.src_state == 1 {
        // 数据库中为开启状态
        on_db = true;
    }

    // api查询摄像头
    let res = app_state.ctx.ana_api.get_source_info(sid.clone()).await;
    if let Err(e) = res {
        error!("error, get_source_info:{}, {:?}", sid, e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let res = res.unwrap();
    if res.code == 0 {
        // 底层存在该摄像头
        on_api = true;
    }

    if state == 1 {
        //开启操作
        if on_db {
            if on_api {
                // 无操作
            } else {
                will_api_create = true;
            }
        } else {
            if on_api {
                will_db_update = true;
                will_api_update = true;
            } else {
                will_db_update = true;
                will_api_create = true;
            }
        }
    } else {
        //关闭操作
        if on_db {
            if on_api {
                will_db_update = true;
                will_api_delete = true;
            } else {
                will_db_update = true;
            }
        } else {
            if on_api {
                will_api_delete = true;
            } else {
                // 无操作
            }
        }
    }


    if will_api_create {
        // 底层创建
        if let Err(e) = create_source_by_api(&app_state.ctx.ana_api, &po).await {
            error!("error, create_source_by_api:{}, {:?}", sid, e);
            return returndata::fail(format!("{:?}", e).as_str());
        }
    }
    if will_api_update {
        // 底层更新
        if let Err(e) = update_source_by_api(&app_state.ctx.ana_api, &po).await {
            error!("error, update_source_by_api:{}, {:?}", sid, e);
            return returndata::fail(format!("{:?}", e).as_str());
        }
    }
    if will_api_delete {
        // 底层删除
        if let Err(e) = delete_source_by_api(&app_state.ctx.ana_api, po.src_sid.as_str()).await {
            error!("error, delete_source_by_api:{}, {:?}", sid, e);
            return returndata::fail(format!("{:?}", e).as_str());
        }
    }

    // 更新数据库
    if will_db_update {
        po.src_state = state as i32;
        po.gmt_modified = Local::now();

        let ctx = app_state.ctx.clone();
        let affect = web::block(move || {
            ctx.web_dao.update_dfsource_for_setstate(&po)
        }).await;
        if let Err(e) = affect {
            error!("error, update_dfsource_for_setstate, {:?}", e);
            return returndata::fail_msg("更新失败", "update fail");
        }
        let affect = affect.unwrap();
        if affect != 1 {
            error!("error, update_dfsource_for_setstate, affect: {}", affect);
            return returndata::fail_msg("更新失败", "update fail");
        }
    }

    returndata::success(SetOnScreenResult {
        src_sid: sid.clone(),
        screen: state,
    })
}

//----------------------------------------
async fn delete_source_by_api(api: &AnalysisApi, src_sid: &str) -> AppResult<()> {
    let res = api.delete_source(src_sid.to_string()).await?;
    if res.code != 0 {
        return Err(AppError::new(res.msg.as_str()));
    }
    Ok(())
}

async fn create_source_by_api(api: &AnalysisApi, po: &CfDfsource) -> AppResult<()> {
    let cfg: CreateSourceReqConfig = serde_json::from_reader(po.src_config.as_bytes())?;

    let res = api.create_source(Some(po.src_sid.clone()),
                                po.src_url.clone(), po.push_url.clone(), cfg).await?;

    if res.code != 0 {
        return Err(AppError::new(res.msg.as_str()));
    }

    Ok(())
}

async fn update_source_by_api(api: &AnalysisApi, po: &CfDfsource) -> AppResult<()> {
    let cfg: CreateSourceReqConfig = serde_json::from_reader(po.src_config.as_bytes())?;

    let res = api.update_source(po.src_sid.clone(),
                                po.src_url.clone(), po.push_url.clone(), cfg).await?;

    if res.code != 0 {
        return Err(AppError::new(res.msg.as_str()));
    }

    Ok(())
}

