use actix_web::{HttpResponse, Result, web};
use log::{debug, error};
use serde::{Deserialize, Serialize};

use cffc_base::model::returndata::{self, ReturnDataType};
use cffc_base::util::utils;

use crate::web::AppState;
use uuid::Uuid;
use chrono::prelude::*;

use actix_web::http::header;
use actix_web::cookie::{Cookie};
use time::Duration;

pub async fn login(app_state: web::Data<AppState>) -> Result<HttpResponse> {
    let mut tpl_ctx = tera::Context::new();
    tpl_ctx.insert("localIp", &app_state.ctx.cfg.local_ip);
    tpl_ctx.insert("port", &app_state.ctx.cfg.http_port);

    let content = app_state.tmpl.render("index.tpl", &tpl_ctx);
    match content {
        Ok(v) => {
            Ok(HttpResponse::Ok().content_type("text/html;charset=utf-8").body(v))
        }
        Err(e) => {
            error!("error, login, {:?}", e);
            Ok(HttpResponse::Ok().content_type("text/html;charset=utf-8").body("template error"))
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogonFormData {
    pub username: Option<String>,
    pub password: Option<String>,
}

fn check_logon_param(form: &web::Form<LogonFormData>) -> (bool, String) {
    let err_msg = "invalid param".to_string();

    if !utils::option_must_length(&form.username, 1, 50) {
        return (false, err_msg);
    }

    if !utils::option_must_length(&form.password, 1, 100) {
        return (false, err_msg);
    }

    (true, "".to_string())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogonResult {
    pub username: String,
    pub token: String,
}

fn build_fail_logonres(msg: &str) -> HttpResponse {
    let data: ReturnDataType<String> = returndata::fail(msg);
    HttpResponse::Ok().json(data.expect_err(""))
}

fn build_succ_logonres(username: &str, token: &str) -> HttpResponse {
    let ck_name = Cookie::build("name", username.to_string()).path("/").finish();
    let ck_token = Cookie::build("token", token.to_string()).path("/").finish();

    let rst = LogonResult {
        username: username.to_string(),
        token: token.to_string(),
    };
    let data = returndata::success(rst);
    HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .set_header(header::SET_COOKIE, ck_name.to_string())
        .set_header(header::SET_COOKIE, ck_token.to_string())
        .json(data.unwrap())
}


//// username
///  password, md5后的值, hex, 小写
///  根据username 查找记录
/// 验证 md5(md5(密码)+salt)
/// 验证通过后，更新beuser记录，设置cookie
pub async fn logon(app_state: web::Data<AppState>, form: web::Form<LogonFormData>)
                   -> HttpResponse {

    // 检测参数
    let (valid, err_msg) = check_logon_param(&form);
    if !valid {
        return build_fail_logonres(err_msg.as_str());
    }

    let username = form.username.as_ref().unwrap();
    let passwd = form.password.as_ref().unwrap().to_lowercase();

    let ctx = app_state.ctx.clone();
    let username_closure = username.clone();
    let po = web::block(move || {
        ctx.web_dao.load_beuser_by_loginname(username_closure.as_str())
    }).await;

    // 发生错误
    if let Err(e) = po {
        error!("error, logon, {:?}", e);
        return build_fail_logonres(format!("error:{:?}", e).as_str());
    }

    let po = po.unwrap();
    // 没有该用户
    if po.is_none() {
        error!("error, logon, can't find beuser:{}", username);
        return build_fail_logonres("invalid username/password");
    }

    let mut po = po.unwrap();
    let passwd_calc = utils::md5_with_salt(passwd.as_str(), po.salt.as_str());
    if !passwd_calc.eq_ignore_ascii_case(po.password.as_str()) {
        // 密码检查不正确
        debug!("logon, invalid username/password:{}, {}", username, passwd);
        return build_fail_logonres("invalid username/password");
    }

    if po.token.is_none() {
        po.token = Some(Uuid::new_v4().to_string());
    }

    let token = po.token.as_ref().unwrap().clone();

    let now = Local::now();
    po.last_login = Some(now);
    po.gmt_modified = now;
    po.ref_count = po.ref_count.map_or(Some(1), |x| Some(x + 1));

    // 更新记录
    let ctx = app_state.ctx.clone();
    let affect = web::block(move || {
        ctx.web_dao.update_beuser_for_logon(&po)
    }).await;

    if let Err(e) = affect {
        error!("error, logon, {:?}", e);
        return build_fail_logonres(format!("error:{:?}", e).as_str());
    }
    let affect = affect.unwrap();
    if affect != 1 {
        error!("error, logon, update beuser, affect: {}", affect);
        return build_fail_logonres(format!("error, update affect:{}", affect).as_str());
    }

    //
    build_succ_logonres(username.as_str(), token.as_str())
}

pub async fn home(app_state: web::Data<AppState>) -> Result<HttpResponse> {
    let mut tpl_ctx = tera::Context::new();
    tpl_ctx.insert("localIp", &app_state.ctx.cfg.local_ip);
    tpl_ctx.insert("port", &app_state.ctx.cfg.http_port);

    let content = app_state.tmpl.render("main.tpl", &tpl_ctx);
    match content {
        Ok(v) => {
            Ok(HttpResponse::Ok().content_type("text/html;charset=utf-8").body(v))
        }
        Err(e) => {
            error!("error, login, {:?}", e);
            Ok(HttpResponse::Ok().content_type("text/html;charset=utf-8").body("template error"))
        }
    }
}

/// 清除cookie

pub async fn logout() -> HttpResponse {
    let ck_name = Cookie::build("name", "".to_string()).max_age(Duration::seconds(0)).path("/").finish();
    let ck_token = Cookie::build("token", "".to_string()).max_age(Duration::seconds(0)).path("/").finish();

    let data = returndata::success("succ".to_string());
    HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .set_header(header::SET_COOKIE, ck_name.to_string())
        .set_header(header::SET_COOKIE, ck_token.to_string())
        .json(data.unwrap())
}