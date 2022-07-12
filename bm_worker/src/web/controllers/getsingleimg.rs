use std::collections::HashMap;

use actix_web::{HttpRequest, HttpResponse, web};
use log::{debug, error};
use tokio::fs;

use cffc_base::model::img_file;

use crate::web::AppState;

fn get_para_str(query: &web::Query<HashMap<String, String>>, para: &str) -> Option<String> {
    if let Some(value) = query.get(para) {
        let v = value.trim();
        if !v.is_empty() {
            return Some(v.to_string());
        }
    }
    None
}

fn get_para_int(query: &web::Query<HashMap<String, String>>, para: &str) -> Option<i64> {
    if let Some(value) = query.get(para) {
        let v = value.trim();
        if !v.is_empty() {
            if let Ok(num) = v.parse::<i64>() {
                return Some(num);
            };
        }
    }
    None
}

/// 人脸记录，cat = 0，id, type= s/l/bg , subid
/// 人脸记录，cat = 1，id, type= s/c , subid
/// 车辆记录, cat = 2，id, type=s/p/bg, subid
fn check_param(query: &web::Query<HashMap<String, String>>) -> (bool, String) {
    let cat = get_para_int(&query, "cat");
    let id = get_para_str(&query, "id");
    let c_type = get_para_str(&query, "type");
    let subid = get_para_int(&query, "subid");

    let mut valid = false;
    let mut err_msg = "invalid paras".to_string();

    if cat.is_none() || id.is_none() {
        return (false, err_msg);
    }

    let cat = cat.unwrap();

    match cat {
        0 => {
            if let Some(v) = c_type {
                if v.eq("s") || v.eq("l") {
                    if subid.is_some() {
                        valid = true;
                    }
                } else if v.eq("bg") {
                    valid = true;
                }
            }
        }
        1 => {
            if let Some(v) = c_type {
                if v.eq("s") {
                    if subid.is_some() {
                        valid = true;
                    }
                } else if v.eq("c") {
                    valid = true;
                }
            }
        }
        2 => {
            if let Some(v) = c_type {
                if v.eq("s") {
                    if subid.is_some() {
                        valid = true;
                    }
                } else if v.eq("p") || v.eq("bg") || v.eq("bin") {
                    valid = true;
                }
            }
        }
        _ => {}
    }
    if valid {
        err_msg = "".to_string();
    }

    (valid, err_msg)
}

fn get_facetrack_img(df_imgs: &str, _cat: i64, id: &str, c_type: &str, subid: Option<i64>) -> String {
    match c_type {
        "s" => {
            img_file::get_facetrack_small_imgpath(df_imgs, id, subid.unwrap()).to_str()
                .map_or("".to_string(), |x| x.to_string())
        }
        "l" => {
            img_file::get_facetrack_large_imgpath(df_imgs, id, subid.unwrap()).to_str()
                .map_or("".to_string(), |x| x.to_string())
        }
        "bg" => {
            img_file::get_facetrack_full_bgpath(df_imgs, id).to_str()
                .map_or("".to_string(), |x| x.to_string())
        }
        _ => unreachable!()
    }
}

fn get_person_img(df_imgs: &str, _cat: i64, id: &str, c_type: &str, subid: Option<i64>) -> String {
    match c_type {
        "s" => {
            img_file::get_person_full_imgpath(df_imgs, id, subid.unwrap()).to_str()
                .map_or("".to_string(), |x| x.to_string())
        }
        "c" => {
            img_file::get_person_full_coverpath(df_imgs, id).to_str()
                .map_or("".to_string(), |x| x.to_string())
        }
        _ => unreachable!()
    }
}

fn get_cartrack_img(df_imgs: &str, _cat: i64, id: &str, c_type: &str, subid: Option<i64>) -> String {
    match c_type {
        "s" => {
            img_file::get_cartrack_full_imgpath(df_imgs, id, subid.unwrap()).to_str()
                .map_or("".to_string(), |x| x.to_string())
        }
        "p" => {
            img_file::get_caretrack_full_platepath(df_imgs, id).to_str()
                .map_or("".to_string(), |x| x.to_string())
        }
        "bg" => {
            img_file::get_cartrack_full_bgpath(df_imgs, id).to_str()
                .map_or("".to_string(), |x| x.to_string())
        }
        "bin" => {
            img_file::get_caretrack_full_platebinary_path(df_imgs, id).to_str()
                .map_or("".to_string(), |x| x.to_string())
        }
        _ => unreachable!()
    }
}


pub async fn get(app_state: web::Data<AppState>, query: web::Query<HashMap<String, String>>, req: HttpRequest) -> HttpResponse {
    /*
    	cat := ctl.getCleanString("cat")
	ctype := ctl.getCleanString("type")
	id := ctl.getCleanString("id")
	subId := ctl.getCleanString("subid")
     */

    debug!("getsingleimg: {}", req.query_string());
    let (valid, _) = check_param(&query);

    if !valid {
        // 参数 无效
        error!("error, getsingleimg, invalid para: {}", req.query_string());
        return HttpResponse::NotFound().body(req.query_string().to_string());
    }

    let cat = get_para_int(&query, "cat").unwrap();
    let id = get_para_str(&query, "id").unwrap();
    let c_type = get_para_str(&query, "type").unwrap();
    let subid = get_para_int(&query, "subid");

    let df_imgs = app_state.ctx.cfg.df_imgs.as_str();

    let path = match cat {
        0 => get_facetrack_img(df_imgs, cat, id.as_str(), c_type.as_str(), subid),
        1 => get_person_img(df_imgs, cat, id.as_str(), c_type.as_str(), subid),
        2 => get_cartrack_img(df_imgs, cat, id.as_str(), c_type.as_str(), subid),
        _ => unreachable!()
    };

    let content = fs::read(path.as_str()).await;
    match content {
        Ok(v) => {
            HttpResponse::Ok().content_type("image/jpeg").body(v)
        }
        Err(e) => {
            error!("error, getsingleimg, read:{}, {:?}", path, e);
            HttpResponse::NotFound().finish()
        }
    }
}