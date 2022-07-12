use std::collections::HashMap;
use std::ffi::OsString;

use actix_web::web;
use chrono::prelude::*;
use log::{debug, warn, error};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use cffc_base::api::bm_api::{ApiFeatureQuality, RecognitionApi};
use cffc_base::model::img_file;
use cffc_base::model::returndata::{self, ReturnDataType};
use cffc_base::util::utils;

use crate::dao::model::{CfDfdb, CfPoi};
use crate::error::{AppError, AppResult};
use crate::web::AppState;
use crate::web::proto::{self, poi::{ImgPathScore, PoiBo}};
use crate::web::proto::poi::ImgAppendItem;
use crate::web::svc::poi_svc;

pub async fn group_list(app_state: web::Data<AppState>) -> ReturnDataType<Vec<CfDfdb>> {
    let ctx = app_state.ctx.clone();
    let list = web::block(move || {
        ctx.web_dao.get_dfdb_list_for_poi()
    }).await;

    if let Err(e) = list {
        error!("error, poi_ctl, get_dfdb_list_for_poi, {:?}", e);
        return returndata::fail("query db fail");
    }
    let list = list.unwrap();
    returndata::success(list)
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
                    form: web::Query<DetailFormData>) -> ReturnDataType<PoiBo> {
    if let Err(e) = check_detail_param(&form) {
        return returndata::fail(e.as_str());
    }

    let sid = form.sid.as_ref().unwrap();

    // 查找poi
    let ctx = app_state.ctx.clone();
    let po_sid = sid.clone();
    let po = web::block(move || {
        ctx.web_dao.load_cfpoi_by_sid(po_sid.as_str())
    }).await;
    if let Err(e) = po {
        error!("error, poi_ctl, load_cfpoi_by_sid:{}, {:?}", sid, e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let po = po.unwrap();
    if po.is_none() {
        error!("error, poi_ctl, can't find poi:{}", sid);
        return returndata::fail(format!("can't find poi: {}", sid).as_str());
    }
    let po = po.unwrap();

    let ctx = app_state.ctx.clone();
    let db_list = web::block(move || {
        ctx.web_dao.get_dfdb_list()
    }).await;
    if let Err(e) = db_list {
        error!("error, poi_ctl, get_dfdb_list:{}, {:?}", sid, e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let db_list = db_list.unwrap();

    let bo = poi_svc::to_bo(&po, &db_list, app_state.ctx.cfg.dfimg_url.as_str());
    returndata::success(bo)
}


//----------------- list -------------------------------
#[derive(Serialize, Deserialize, Debug)]
pub struct ListResult {
    pub page: proto::DataPage,
    pub list: Vec<PoiBo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListFormData {
    #[serde(rename = "pageSize")]
    pub page_size: Option<String>,

    #[serde(rename = "pageNo")]
    pub page_no: Option<String>,

    pub name: Option<String>,

    #[serde(rename = "identityCard")]
    pub identity_card: Option<String>,

    pub gender: Option<String>,
    pub group: Option<String>,
    pub threshold: Option<String>,
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

    if !utils::option_should_num_range(&form.threshold, 1, 99) {
        return Err("invalid threshold".to_string());
    }

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
    let identity_card = utils::clean_option_string(&form.identity_card);
    let gender = utils::get_option_num(&form.gender);
    let group = utils::clean_option_string(&form.group);
    let threshold = utils::get_option_num(&form.threshold);

    debug!("page_size:{}", page_size);
    debug!("page_no:{}", page_no);
    debug!("name:{:?}", name);
    debug!("identity_card:{:?}", identity_card);
    debug!("gender:{:?}", gender);
    debug!("group:{:?}", group);
    debug!("threshold:{:?}", threshold);


    // 查询 db list
    let ctx = app_state.ctx.clone();
    let db_list = web::block(move || {
        ctx.web_dao.get_dfdb_list()
    }).await;
    if let Err(e) = db_list {
        error!("error, poi_ctl, get_dfdb_list, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let db_list = db_list.unwrap();


    // 查询总数
    let ctx = app_state.ctx.clone();
    let name_cl = name.clone();
    let identity_card_cl = identity_card.clone();
    let gender_cl = gender.clone();
    let group_cl = group.clone();
    let threshold_cl = threshold.clone();
    let total = web::block(move || {
        ctx.web_dao.get_poi_total(name_cl, identity_card_cl, gender_cl,
                                  group_cl, threshold_cl)
    }).await;
    if let Err(e) = total {
        error!("error, poi_ctl, get_poi_total, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let total = total.unwrap();
    if total.is_none() {
        error!("error, poi_ctl, get_poi_total return null");
        return returndata::fail("can't get total");
    }
    let total = total.unwrap();

    // 查询分页数据
    let dp = proto::DataPage::new(total as u64,
                                  page_size as u64, page_no as u64);
    let ctx = app_state.ctx.clone();
    let name_cl = name.clone();
    let identity_card_cl = identity_card.clone();
    let gender_cl = gender.clone();
    let group_cl = group.clone();
    let threshold_cl = threshold.clone();
    let start_index = dp.get_start_index();
    let poi_list = web::block(move || {
        ctx.web_dao.get_poi_datapage(name_cl, identity_card_cl, gender_cl,
                                     group_cl, threshold_cl,
                                     page_size, start_index as i64)
    }).await;
    if let Err(e) = poi_list {
        error!("error, poi_ctl, get_poi_datapage, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let poi_list = poi_list.unwrap();
    let bo_list = poi_svc::to_bo_list(&poi_list, &db_list, app_state.ctx.cfg.dfimg_url.as_str());

    returndata::success(ListResult {
        page: dp,
        list: bo_list,
    })
}

//----------------- add -------------------------------

fn check_add_param(form: &web::Form<HashMap<String, String>>) -> std::result::Result<(), String> {
    // 必填
    let img_count = form.get("imgCount").map(|x| x.clone());
    if !utils::option_must_num(&img_count) {
        return Err("invalid imgCount".to_string());
    }

    let name = form.get("name").map(|x| x.clone());
    if !utils::option_must_length(&name, 1, 50) {
        return Err("invalid name".to_string());
    }

    let gender = form.get("gender").map(|x| x.clone());
    if !utils::option_must_num_range(&gender, 1, 2) {
        return Err("invalid gender".to_string());
    }

    let group = form.get("group").map(|x| x.clone());
    if !utils::option_must_length(&group, 1, 50) {
        return Err("invalid group".to_string());
    }

    let threshold = form.get("threshold").map(|x| x.clone());
    if !utils::option_must_num_range(&threshold, 1, 99) {
        return Err("invalid threshold".to_string());
    }

    //关联验证
    let img_count = utils::get_option_must_num(&img_count);
    for i in 1..img_count + 1 {
        let img_n = form.get(format!("img_{}", i).as_str()).map(|x| x.clone());
        let img_score_n = form.get(format!("imgScore_{}", i).as_str()).map(|x| x.clone());

        if !utils::option_must_float(&img_score_n) {
            return Err(format!("invalid imgScore_{}", i));
        }

        if !utils::option_must_length(&img_n, 1, 200) {
            return Err(format!("invalid img_{}", i));
        }
    }

    let cover_index = form.get("coverIndex").map(|x| x.clone());
    if !utils::option_must_num_range(&cover_index, 1, img_count) {
        return Err("invalid coverIndex".to_string());
    }

    Ok(())
}

pub async fn add(app_state: web::Data<AppState>, form: web::Form<HashMap<String, String>>) -> ReturnDataType<String> {
    debug!("poi_ctl, add, map: {:?}", form);

    //检查参数
    if let Err(e) = check_add_param(&form) {
        return returndata::fail(format!("{}", e).as_str());
    }

    //读取参数
    let name = form.get("name").unwrap().clone();
    let group = form.get("group").unwrap().clone();
    let cover_index = utils::get_option_must_num(&form.get("coverIndex").map(|x| x.clone()));
    let gender = utils::get_option_must_num(&form.get("gender").map(|x| x.clone()));
    let threshold = utils::get_option_must_num(&form.get("threshold").map(|x| x.clone()));
    let img_count = utils::get_option_must_num(&form.get("imgCount").map(|x| x.clone()));
    let identity_card = form.get("identityCard").map(|x| x.clone());
    let now = Local::now();

    let mut pathscore_list = Vec::new();
    for i in 1..img_count + 1 {
        let img_n = form.get(format!("img_{}", i).as_str()).map(|x| x.clone());
        let img_score_n = form.get(format!("imgScore_{}", i).as_str()).map(|x| x.clone());

        let img_n = utils::clean_relate_path(&img_n.unwrap());
        let img_score_n = utils::get_option_float(&img_score_n).unwrap();
        pathscore_list.push(ImgPathScore {
            path: img_n,
            score: img_score_n,
        });
    }

    // api获取特征值
    let root = app_state.ctx.cfg.web.upload_path.as_str();
    let mut img_content_list = Vec::new();
    for v in pathscore_list.iter() {
        let img_path = img_file::get_upload_full_path(root, v.path.as_str());
        let content = utils::read_file_base64(&img_path).await;
        if let Err(e) = content {
            error!("error, poi_ctl, read_file_base64:{}, {:?}", img_path, e);
            return returndata::fail(format!("{:?}", e).as_str());
        }
        let content = content.unwrap();
        img_content_list.push(content);
    }
    let res = app_state.ctx.recg_api.get_features(img_content_list, false).await;
    if let Err(e) = res {
        error!("error, poi_ctl, get_features, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let res = res.unwrap();
    if res.code != 0 {
        error!("error, poi_ctl, get_features, return code:{}, msg:{}", res.code, res.msg);
        return returndata::fail(format!("get_features, return code:{}, msg:{}", res.code, res.msg).as_str());
    }
    let img_feature_list = match res.features {
        Some(v) => v,
        None => {
            error!("error, poi_ctl, get_features, features is null");
            return returndata::fail("get_features, features is null");
        }
    };
    if img_feature_list.len() != img_count as usize {
        error!("error, poi_ctl, get_features, return features count not equal, {},{}", img_count, img_feature_list.len());
        return returndata::fail("features count not equal files count");
    }

    let poi_sid = Uuid::new_v4().to_string();

    // 底层 api创建
    let ids = vec![poi_sid.clone()];
    let mut feaqua_list = Vec::new();
    for i in 0..img_count as usize {
        let feature = img_feature_list.get(i).unwrap().clone();
        let quality = pathscore_list.get(i).unwrap().score;
        feaqua_list.push(ApiFeatureQuality {
            feature,
            quality,
        });
    }

    let res = app_state.ctx.recg_api.create_persons(group.clone(), ids, vec![feaqua_list]).await;
    if let Err(e) = res {
        error!("error, poi_ctl, create_persons, {:?}", e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let res = res.unwrap();
    if res.code != 0 {
        error!("error, poi_ctl, create_persons, return code:{}, msg:{}", res.code, res.msg);
        return returndata::fail(format!("create_persons, return code:{}, msg:{}", res.code, res.msg).as_str());
    }
    if res.persons.is_none() {
        error!("error, poi_ctl, create_persons, return persons is null");
        return returndata::fail("create_persons, return persons is null");
    }
    let res_persons = res.persons.unwrap();
    if res_persons.len() != 1 {
        error!("error, poi_ctl, create_persons, return persons, size:{}", res_persons.len());
        return returndata::fail(format!("create_persons, return persons, size:{}", res_persons.len()).as_str());
    }
    let res_person = res_persons.get(0).unwrap();
    if res_person.faces.len() != img_count as usize {
        error!("error, poi_ctl, get_features, faces count not equal, {},{}", res_person.faces.len(), img_count);
        return returndata::fail("faces count not equal files count");
    }

    // 复制图片
    let df_imgs_path = app_state.ctx.cfg.df_imgs.as_str();
    let dir = img_file::get_person_imgdir(df_imgs_path, poi_sid.as_str());
    let created = utils::prepare_dir(&dir).await;
    if let Err(e) = created {
        error!("error, poi_ctl, prepare_dir:{:?}, {:?}", dir, e);
        return returndata::fail("prepare_dir fail");
    }
    let mut img_ids = String::new();
    let upload_root = app_state.ctx.cfg.web.upload_path.as_str();

    for i in 0..img_count as usize {
        let face_id = *res_person.faces.get(i).unwrap();
        let score = pathscore_list.get(i).unwrap().score;
        let relate_imgpath = &pathscore_list.get(i).unwrap().path;
        let dst_file = img_file::get_person_full_imgpath(df_imgs_path, poi_sid.as_str(), face_id);
        let src_file = img_file::get_upload_imgpath_by_relate(upload_root, relate_imgpath);

        img_ids.push_str(&format!("{}:{},", face_id, score));

        if let Err(e) = utils::copy_file(&src_file, &dst_file).await {
            error!("error, poi_ctl, copy_file:{:?},{:?}, {:?}", src_file, dst_file, e);

            // 撤回，删除底层的创建
            let deleted = delete_person_by_api(&app_state.ctx.recg_api, &group, &poi_sid).await;
            if let Err(delete_err) = deleted {
                error!("error, poi_ctl, delete_person_by_api:{}, {:?}", poi_sid, delete_err);
            }

            return returndata::fail("copy_file fail");
        }

        if i + 1 == cover_index as usize {
            // 拷贝封面图
            let dst_file = img_file::get_person_full_coverpath(df_imgs_path, poi_sid.as_str());
            if let Err(e) = utils::copy_file(&src_file, &dst_file).await {
                error!("error, poi_ctl, copy_file:{:?},{:?}, {:?}", src_file, dst_file, e);

                // 撤回，删除底层的创建
                let deleted = delete_person_by_api(&app_state.ctx.recg_api, &group, &poi_sid).await;
                if let Err(delete_err) = deleted {
                    error!("error, poi_ctl, delete_person_by_api:{}, {:?}", poi_sid, delete_err);
                }


                return returndata::fail("copy_file fail");
            }
        }
    }
    if !img_ids.is_empty() {
        // 去掉最后的 ","
        img_ids.truncate(img_ids.len() - 1);
    }

    // 保存数据库
    let po = CfPoi {
        id: 0,
        poi_sid: poi_sid.clone(),
        db_sid: group.clone(),
        name,
        gender: Some(gender as i32),
        identity_card,
        threshold: threshold as i32,
        tp_id: None,
        feature_ids: img_ids,
        cover: Some(1),
        tag: None,
        imp_tag: None,
        memo: None,
        flag: Some(0),
        gmt_create: now,
        gmt_modified: now,
    };

    let ctx = app_state.ctx.clone();
    let poi_id = web::block(move || {
        ctx.web_dao.save_poi(&po)
    }).await;
    if let Err(e) = poi_id {
        error!("error, poi_ctl, save_poi:{}, {:?}", poi_sid, e);
        // 撤回，删除底层的创建
        let deleted = delete_person_by_api(&app_state.ctx.recg_api, &group, &poi_sid).await;
        if let Err(delete_err) = deleted {
            error!("error, poi_ctl, delete_person_by_api:{}, {:?}", poi_sid, delete_err);
        }
        return returndata::fail("save_poi fail");
    }

    let poi_id = poi_id.unwrap();
    debug!("poi_ctl, create poi ok, {}, {}", poi_sid, poi_id);
    returndata::success_str(poi_sid.as_str())
}

async fn delete_person_by_api(client: &RecognitionApi, db_sid: &str, poi_sid: &str) -> AppResult<()> {
    let res = client.delete_person(db_sid.to_string(), poi_sid.to_string()).await?;
    if res.code != 0 {
        return Err(AppError::new(&format!("delete_person, return code:{}, msg:{}", res.code, res.msg)));
    }

    Ok(())
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

/// 先删除底层，再删除数据库,删除目录
pub async fn delete(app_state: web::Data<AppState>, form: web::Form<DeleteFormData>) -> ReturnDataType<String> {
    if let Err(e) = check_delete_param(&form) {
        return returndata::fail(e.as_str());
    }

    let sid = form.sids.as_ref().unwrap();

    let ctx = app_state.ctx.clone();
    let poi_sid = sid.clone();
    let po = web::block(move || {
        ctx.web_dao.load_cfpoi_by_sid(&poi_sid)
    }).await;
    if let Err(e) = po {
        error!("error, poi_ctl, load_cfpoi_by_sid:{}, {:?}", sid, e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let po = po.unwrap();
    if po.is_none() {
        error!("error, poi_ctl, person not exsit, {}", sid);
        return returndata::fail("person not exsit");
    }
    let po = po.unwrap();

    // api 删除
    let res = app_state.ctx.recg_api.delete_person(po.db_sid.clone(), sid.clone()).await;
    if let Err(e) = res {
        error!("error, poi_ctl, delete_person:{}, {:?}", sid, e);
    } else {
        let res = res.unwrap();
        debug!(" poi_ctl, api delete_person:{}, return code:{}, msg:{}", sid, res.code, res.msg);
    }

    // 数据库删除
    let ctx = app_state.ctx.clone();
    let poi_sid = sid.clone();
    let affect = web::block(move || {
        ctx.web_dao.delete_cfpoi_by_sid(&poi_sid)
    }).await;
    if let Err(e) = affect {
        error!("error, poi_ctl, delete_cfpoi_by_sid, {:?}", e);
    } else {
        let affect = affect.unwrap();
        debug!("poi_ctl, delete_cfpoi_by_sid:{}, affect:{}", sid, affect);
    }

    // 图片目录删除
    let df_imgs_path = app_state.ctx.cfg.df_imgs.as_str();
    let dir = img_file::get_person_imgdir(df_imgs_path, sid.as_str());
    let deleted = utils::remove_dir(&dir).await;
    if let Err(e) = deleted {
        error!("error, poi_ctl, remove_dir:{:?}, {:?}", dir, e);
    }

    debug!("poi_ctl, delete person:{}, ok, {}", po.name, po.poi_sid);

    returndata::success_str("succ")
}


//----------------- modify -------------------------------


fn check_modify_param(form: &web::Form<HashMap<String, String>>) -> std::result::Result<(), String> {
    // 必填
    let img_count = form.get("imgCount").map(|x| x.clone());
    if !utils::option_must_num(&img_count) {
        return Err("invalid imgCount".to_string());
    }

    let name = form.get("name").map(|x| x.clone());
    if !utils::option_must_length(&name, 1, 50) {
        return Err("invalid name".to_string());
    }

    let sid = form.get("sid").map(|x| x.clone());
    if !utils::option_must_length(&sid, 1, 50) {
        return Err("invalid sid".to_string());
    }

    let gender = form.get("gender").map(|x| x.clone());
    if !utils::option_must_num_range(&gender, 1, 2) {
        return Err("invalid gender".to_string());
    }

    let threshold = form.get("threshold").map(|x| x.clone());
    if !utils::option_must_num_range(&threshold, 1, 99) {
        return Err("invalid threshold".to_string());
    }

    //关联验证
    let img_count = utils::get_option_must_num(&img_count);
    for i in 1..img_count + 1 {
        let img_n = form.get(format!("img_{}", i).as_str()).map(|x| x.clone());
        let img_id_n = form.get(format!("imgId_{}", i).as_str()).map(|x| x.clone());
        let img_score_n = form.get(format!("imgScore_{}", i).as_str()).map(|x| x.clone());

        if !utils::option_must_length(&img_n, 1, 200) {
            return Err(format!("invalid img_{}", i));
        }
        if !utils::option_must_num(&img_id_n) {
            return Err(format!("invalid imgId_{}", i));
        }
        if !utils::option_must_float(&img_score_n) {
            return Err(format!("invalid imgScore_{}", i));
        }
    }

    let cover_index = form.get("coverIndex").map(|x| x.clone());
    if !utils::option_should_num_range(&cover_index, 1, img_count) {
        return Err("invalid coverIndex".to_string());
    }

    Ok(())
}

/**
img_xxx         必填  新图片相对路径 (裁剪接口返回的值）xxx从1开始编号，旧人脸该字段设置为人脸faceId
imgId_xxx       必填  人脸faceId, 新图片，该字段设置为-1
imgScore_xx     必填  图片quality

    "name": "123",
    "sid": "582b6a24-d61b-4921-a3a1-ababfd0131c6",
    "coverIndex": "",
    "gender": "1"
    "threshold": "70",
    "imgCount": "2",
    "identityCard": "652323198707244130",

    "group": "56e6a47c-3d4d-4f99-b6a3-ca24028358df",
    "cover_url": "",

    "img_2": "108",
    "imgId_2": "108",
    "imgScore_2": "0.9999998807907104",

    "img_1": "/crop/2020/12/16/1608106312618_1.jpg",
    "imgId_1": "-1",
    "imgScore_1": "0.9999344348907471",



*/

/// 对新增图片进行特征提取feature
/// 将新的特征加入到person中，复制图片
/// 将去掉的faceid从person中移除，删除本地图片
/// 更新数据库
pub async fn modify(app_state: web::Data<AppState>, form: web::Form<HashMap<String, String>>) -> ReturnDataType<String> {
    debug!("poi_ctl, add, map: {:?}", form);
    if let Err(e) = check_modify_param(&form) {
        return returndata::fail(format!("{}", e).as_str());
    }

    //读取参数
    let name = form.get("name").unwrap().clone();
    let sid = form.get("sid").unwrap().clone();
    let cover_index = utils::get_option_num(&form.get("coverIndex").map(|x| x.clone()));
    let gender = utils::get_option_must_num(&form.get("gender").map(|x| x.clone()));
    let threshold = utils::get_option_must_num(&form.get("threshold").map(|x| x.clone()));
    let img_count = utils::get_option_must_num(&form.get("imgCount").map(|x| x.clone()));
    let identity_card = form.get("identityCard").map(|x| x.clone());
    let now = Local::now();

    let cover_index = cover_index.map_or(-1, |x| x);
    debug!("poi_ctl, add, cover_index: {}", cover_index);
    let mut cover_src_path = OsString::new(); // 存放cover的图片路径，用于后续复制

    let mut img_append_list = Vec::new();   // 新增加的图片信息
    let mut img_remain_list = Vec::new();   // 原来的，留下来的 faceid
    for i in 1..img_count + 1 {
        let img_n = form.get(format!("img_{}", i).as_str()).map(|x| x.clone());
        let img_id_n = form.get(format!("imgId_{}", i).as_str()).map(|x| x.clone());
        let img_score_n = form.get(format!("imgScore_{}", i).as_str()).map(|x| x.clone());

        let img_n = utils::clean_relate_path(&img_n.unwrap());
        let img_id_n = utils::get_option_num(&img_id_n).unwrap();
        let img_score_n = utils::get_option_float(&img_score_n).unwrap();

        if img_id_n == -1 {
            // 新图片
            img_append_list.push(ImgAppendItem {
                path: img_n.clone(),
                score: img_score_n,
                feature: "".to_string(),
                face_id: 0,
            });
        } else {
            // 原来留下来的
            img_remain_list.push((img_id_n, img_score_n));
        }

        //获取cover img path
        if cover_index == i {
            if img_id_n == -1 {
                // 新图片
                cover_src_path = img_file::get_upload_full_path(&app_state.ctx.cfg.web.upload_path, &img_n).into();
            } else {
                cover_src_path = img_file::get_person_full_imgpath(&app_state.ctx.cfg.df_imgs, &sid, img_id_n);
            }
        }
    }

    // 查询person
    let ctx = app_state.ctx.clone();
    let poi_sid = sid.clone();
    let po = web::block(move || {
        ctx.web_dao.load_cfpoi_by_sid(&poi_sid)
    }).await;
    if let Err(e) = po {
        error!("error, poi_ctl, load_cfpoi_by_sid:{}, {:?}", sid, e);
        return returndata::fail(format!("{:?}", e).as_str());
    }
    let po = po.unwrap();
    if po.is_none() {
        error!("error, poi_ctl, person not exsit, {}", sid);
        return returndata::fail("person not exsit");
    }
    let mut po = po.unwrap();
    let df_imgs_path = app_state.ctx.cfg.df_imgs.as_str();

    // 如果有新增图片,对新图片进行处理（提取特征，添加，复制）
    if img_append_list.len() > 0 {
        let upload_root = app_state.ctx.cfg.web.upload_path.as_str();
        let mut img_content_list = Vec::new();
        for v in img_append_list.iter() {
            let img_path = img_file::get_upload_full_path(&upload_root, v.path.as_str());
            let content = utils::read_file_base64(&img_path).await;
            if let Err(e) = content {
                error!("error, poi_ctl, read_file_base64:{}, {:?}", img_path, e);
                return returndata::fail(format!("{:?}", e).as_str());
            }
            let content = content.unwrap();
            img_content_list.push(content);
        }

        let res = app_state.ctx.recg_api.get_features(img_content_list, false).await;
        if let Err(e) = res {
            error!("error, poi_ctl, get_features, {:?}", e);
            return returndata::fail(format!("{:?}", e).as_str());
        }
        let res = res.unwrap();
        if res.code != 0 {
            error!("error, poi_ctl, get_features, return code:{}, msg:{}", res.code, res.msg);
            return returndata::fail(format!("get_features, return code:{}, msg:{}", res.code, res.msg).as_str());
        }
        let img_feature_list = match res.features {
            Some(v) => v,
            None => {
                error!("error, poi_ctl, get_features, features is null");
                return returndata::fail("get_features, features is null");
            }
        };
        if img_feature_list.len() != img_append_list.len() {
            error!("error, poi_ctl, get_features, return features count not equal, {},{}", img_feature_list.len(), img_append_list.len());
            return returndata::fail("features count not equal files count");
        }

        for i in 0..img_feature_list.len() {
            let item = img_append_list.get_mut(i).unwrap();
            item.feature = img_feature_list.get(i).unwrap().clone();
        }

        // 将新特征值加入到person中
        let mut feaqua_list = Vec::new();
        for v in img_append_list.iter() {
            feaqua_list.push(ApiFeatureQuality {
                feature: v.feature.clone(),
                quality: v.score,
            });
        }
        let res = app_state.ctx.recg_api.add_features_to_person(po.db_sid.clone(), sid.clone(), feaqua_list).await;
        if let Err(e) = res {
            error!("error, poi_ctl, add_features_to_person, {:?}", e);
            return returndata::fail(format!("{:?}", e).as_str());
        }
        let res = res.unwrap();
        if res.code != 0 {
            error!("error, poi_ctl, add_features_to_person, return code:{}, msg:{}", res.code, res.msg);
            return returndata::fail(format!("add_features_to_person, return code:{}, msg:{}", res.code, res.msg).as_str());
        }
        if res.ids.is_none() {
            error!("error, poi_ctl, add_features_to_person, return ids is null");
            return returndata::fail("add_features_to_person, return ids is null");
        }
        let res_ids = res.ids.unwrap();
        if res_ids.len() != img_append_list.len() {
            error!("error, poi_ctl, add_features_to_person, return ids count not equal, {},{}", res_ids.len(), img_append_list.len());
            return returndata::fail("return ids count not equal");
        }
        for i in 0..res_ids.len() {
            let item = img_append_list.get_mut(i).unwrap();
            item.face_id = *res_ids.get(i).unwrap();
        }

        // 复制图片

        let dir = img_file::get_person_imgdir(df_imgs_path, sid.as_str());
        let created = utils::prepare_dir(&dir).await;
        if let Err(e) = created {
            error!("error, poi_ctl, prepare_dir:{:?}, {:?}", dir, e);
            return returndata::fail("prepare_dir fail");
        }

        let upload_root = app_state.ctx.cfg.web.upload_path.as_str();

        for i in 0..img_append_list.len() {
            let face_id = img_append_list.get(i).unwrap().face_id;
            // let score = img_append_list.get(i).unwrap().score;
            let relate_imgpath = &img_append_list.get(i).unwrap().path;
            let dst_file = img_file::get_person_full_imgpath(df_imgs_path, sid.as_str(), face_id);
            let src_file = img_file::get_upload_imgpath_by_relate(upload_root, relate_imgpath);

            if let Err(e) = utils::copy_file(&src_file, &dst_file).await {
                error!("error, poi_ctl, copy_file:{:?},{:?}, {:?}", src_file, dst_file, e);
                return returndata::fail("copy_file fail");
            }
        }
    } // 如果有新增图片,对新图片进行处理


    if cover_index != -1 {
        // 拷贝封面图
        let dst_file = img_file::get_person_full_coverpath(df_imgs_path, sid.as_str());
        if let Err(e) = utils::copy_file(&cover_src_path, &dst_file).await {
            error!("error, poi_ctl, copy_file:{:?},{:?}, {:?}", cover_src_path, dst_file, e);
            return returndata::fail("copy_file fail");
        }
    } else {
        // 删除封面图
        let dst_file = img_file::get_person_full_coverpath(df_imgs_path, sid.as_str());
        if let Err(e) = utils::remove_file(&dst_file).await {
            // 只是 warn 记录
            warn!("warn, poi_ctl, remove_file:{:?}, {:?}", dst_file, e);
        }
    }

    // 去掉不存在的faceid
    let old_faceids = img_file::get_item_from_idscores(&po.feature_ids);
    if let Err(e) = old_faceids {
        error!("error, poi_ctl, get_item_from_idscores:{}, {:?}", po.feature_ids, e);
        return returndata::fail("get_item_from_idscores fail");
    }
    let old_faceids = old_faceids.unwrap();

    let img_missing_list: Vec<i64> = old_faceids.iter().filter_map(|x| {
        let find = img_remain_list.iter().any(|y| x.0 == y.0);
        if find {
            None
        } else {
            Some(x.0)
        }
    }).collect();

    for v in img_missing_list.iter() {
        // api 删除feature , 删除对应图片文件
        let res = app_state.ctx.recg_api.delete_person_feature(po.db_sid.clone(), po.poi_sid.clone(), *v).await;
        if let Err(e) = res {
            error!("error, poi_ctl, delete_person_feature:{}, {:?}", po.poi_sid, e);
            return returndata::fail("delete_person_feature fail");
        }
        let res = res.unwrap();
        if res.code != 0 {
            error!("error, poi_ctl, delete_person_feature, return code:{}, msg:{}", res.code, res.msg);
            return returndata::fail(format!("delete_person_feature, return code:{}, msg:{}", res.code, res.msg).as_str());
        }

        let img_path = img_file::get_person_full_imgpath(&app_state.ctx.cfg.df_imgs, &po.poi_sid, *v);
        if let Err(e) = utils::remove_file(&img_path).await {
            error!("error, poi_ctl, remove_file:{:?}, {:?}", img_path, e);
        };
    }

    //更新数据库
    let mut img_ids = String::new();
    for v in img_append_list.iter() {
        img_ids.push_str(&format!("{}:{},", v.face_id, v.score));
    }
    for v in img_remain_list.iter() {
        img_ids.push_str(&format!("{}:{},", v.0, v.1));
    }
    if !img_ids.is_empty() {
        // 去掉最后的 ","
        img_ids.truncate(img_ids.len() - 1);
    }
    po.feature_ids = img_ids;
    po.name = name;
    po.gender = Some(gender as i32);
    po.threshold = threshold as i32;
    po.identity_card = identity_card;
    po.gmt_modified = now;
    if cover_index != -1 {
        po.cover = Some(1);
    } else {
        po.cover = Some(0);
    }

    let ctx = app_state.ctx.clone();
    let affect = web::block(move || {
        ctx.web_dao.update_cfpoi_for_modify(&po)
    }).await;
    if let Err(e) = affect {
        error!("error, poi_ctl, update_cfpoi_for_modify:{}, {:?}", sid.clone(), e);
        return returndata::fail("update_cfpoi_for_modify fail");
    }
    let affect = affect.unwrap();
    if affect != 1 {
        error!("error, poi_ctl, update_cfpoi_for_modify, affect:{}", affect);
        return returndata::fail(&format!("update_cfpoi_for_modify, affect:{}", affect));
    }

    returndata::success_str("succ")
}