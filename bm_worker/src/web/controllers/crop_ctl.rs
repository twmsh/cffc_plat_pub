use actix_multipart::Multipart;
use actix_web::{http, HttpResponse, web};
use bytes::Buf;
use chrono::prelude::*;
use log::{debug, error};
use serde::{Deserialize, Serialize};

use cffc_base::api::bm_api::DetectRes;
use cffc_base::model::img_file;
use cffc_base::model::returndata::{self, ReturnDataType};
use cffc_base::util::multipart_form;
use cffc_base::util::utils;

use crate::error::{AppError, AppResult};
use crate::web::AppState;

//----------------- const ------------------------------------
const CROP_MODULE: &str = "crop";

//----------------- do_options -------------------------------
pub async fn do_options() -> HttpResponse {
    HttpResponse::Ok()
        .set_header(http::header::ACCESS_CONTROL_ALLOW_METHODS, "POST, OPTIONS")
        .set_header(http::header::ACCESS_CONTROL_ALLOW_HEADERS, "x-requested-with")
        .finish()
}

//----------------- crop -------------------------------
#[derive(Serialize, Deserialize, Debug)]
pub struct CropResultItem {
    pub path: String,
    pub url: String,
    pub score: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CropResult {
    pub path: String,
    pub url: String,
    pub items: Vec<CropResultItem>,
}

/// 解析multipart
/// 准备目录，保存上层图片
/// api裁剪，保存小图片
/// 返回结果
pub async fn crop(app_state: web::Data<AppState>, payload: Multipart) -> ReturnDataType<CropResult> {
    let values = multipart_form::parse_multi_form(payload).await;
    if let Err(e) = values {
        error!("error, crop_ctl, parse_multi_form, {}", e);
        return returndata::fail(e.as_str());
    }

    let now = Local::now();
    let values = values.unwrap();
    let img = values.get_file_value("img");
    if img.is_none() {
        error!("error, crop_ctl, img is none");
        return returndata::fail("invalid img");
    }
    let (file_name, file_content) = img.unwrap();
    debug!("crop_ctl, recv upload, {}, {}", file_name, file_content.len());


    // 准备目录
    let root = app_state.ctx.cfg.web.upload_path.clone();
    let url_prefix = app_state.ctx.cfg.web.upload_url.clone();
    let relate_path = img_file::get_upload_relate_path(CROP_MODULE, now);
    let dir = img_file::get_upload_full_path(root.as_str(), relate_path.as_str());
    let created = utils::prepare_dir(&dir).await;
    if let Err(e) = created {
        error!("error, crop_ctl, prepare_dir:{}, {:?}", dir, e);
        return returndata::fail("prepare_dir fail");
    }

    //保存上传图原始图
    let img_fn = format!("{}.{}", now.timestamp_millis(), "jpg");
    let img_path = img_file::get_upload_img_path(&dir, &img_fn);
    if let Err(e) = utils::write_file(&img_path, file_content.bytes()).await {
        error!("error, crop_ctl, save:{:?}, {:?}", img_path, e);
        return returndata::fail("save file fail");
    }
    debug!("crop_ctl, save img: {:?}", img_path);

    // api 裁剪
    let img_base64 = base64::encode(file_content.bytes());
    let res = app_state.ctx.recg_api.detect(img_base64, false, false).await;
    if let Err(e) = res {
        error!("error, crop_ctl, api detect, {:?}", e);
        return returndata::fail("detect img fail");
    }
    let res = res.unwrap();
    if res.code != 0 {
        error!("error, crop_ctl, api detect, return code:{}, msg:{}", res.code, res.msg);
        return returndata::fail(format!("detect return code:{}, msg:{}", res.code, res.msg).as_str());
    }

    // 保存小图
    let items = save_faces(res, root.as_str(),
                           app_state.ctx.cfg.web.upload_url.as_str(), relate_path.as_str(), now.timestamp_millis()).await;
    if let Err(e) = items {
        error!("error, crop_ctl, save_faces, {:?}", e);
        return returndata::fail("save_faces fail");
    }
    let items = items.unwrap();

    returndata::success(CropResult {
        path: format!("{}/{}", relate_path, img_fn),
        url: format!("{}/{}/{}", url_prefix, relate_path, img_fn),
        items,
    })
}

async fn save_faces(res: DetectRes, root: &str, url_prefix: &str, relate_path: &str, ts: i64) -> AppResult<Vec<CropResultItem>> {
    if res.faces.is_none() {
        return Ok(vec![]);
    }

    let mut items = Vec::new();
    let faces = res.faces.unwrap();
    for (i, v) in faces.iter().enumerate() {
        let img_fn = format!("{}_{}.jpg", ts, i + 1);
        let file_path = img_file::get_upload_img_path(&format!("{}/{}", root, relate_path), &img_fn);
        let saved = utils::write_file_base64(&file_path, v.aligned.as_str()).await;
        if let Err(e) = saved {
            error!("error, crop_ctl, write_file_base64, {:?}, {:?}", file_path, e);
            return Err(AppError::from_debug(e));
        }
        debug!("crop_ctl, {}, save face: {:?}", i, file_path);
        items.push(CropResultItem {
            path: format!("{}/{}", relate_path, img_fn),
            url: format!("{}/{}/{}", url_prefix, relate_path, img_fn),
            score: v.score,
        });
    }

    Ok(items)
}