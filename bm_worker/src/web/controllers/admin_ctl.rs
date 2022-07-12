use actix_web::{HttpRequest, web};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

use cffc_base::model::returndata::{self, ReturnDataType};
use cffc_base::util::utils;

use crate::dao::model::BeUser;
use crate::web::proto::admin::BeuserBo;
use uuid::Uuid;
use crate::web::AppState;

use log::{error};

pub async fn detail(req: HttpRequest) -> ReturnDataType<BeuserBo> {
    if let Some(po) = req.extensions_mut().get::<BeUser>() {
        returndata::success(BeuserBo {
            id: po.id,
            name: utils::unwarp_option_string(&po.name, ""),
            login_name: po.login_name.clone(),
            phone: utils::unwarp_option_string(&po.phone, ""),
            email: utils::unwarp_option_string(&po.email, ""),
            service_flag: po.service_flag.unwrap_or(1),
            last_login: utils::get_option_datetime(&po.last_login, Local::now()),
            memo: utils::unwarp_option_string(&po.memo, ""),
            gmt_create: po.gmt_create,
            gmt_modified: po.gmt_modified,
        })
    } else {
        returndata::fail("not find user")
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ModifyFormData {
    pub name: Option<String>,

    #[serde(rename(deserialize = "oldPasswd"))]
    pub old_passwd: Option<String>,

    #[serde(rename(deserialize = "newPasswd"))]
    pub new_passwd: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

fn check_modify_param(form: &web::Form<ModifyFormData>) -> std::result::Result<(), String> {
    if !utils::option_must_length(&form.name, 1, 50) {
        return Err("invalid name".to_string());
    }

    if !utils::option_must_length(&form.old_passwd, 1, 100) {
        return Err("invalid oldPasswd".to_string());
    }

    if !utils::option_must_length(&form.new_passwd, 1, 100) {
        return Err("invalid newPasswd".to_string());
    }

    Ok(())
}

/// 检查 extensions 有没有 po
/// 判断 旧密码是否正确
/// 计算新密码+salt，更新token
/// 更新数据库记录
pub async fn modify(app_state: web::Data<AppState>, req: HttpRequest, form: web::Form<ModifyFormData>) -> ReturnDataType<String> {

    //检查参数
    if let Err(e) = check_modify_param(&form) {
        return returndata::fail(e.as_str());
    }

    let username = form.name.as_ref().unwrap();
    let old_passwd = form.old_passwd.as_ref().unwrap();
    let new_passwd = form.new_passwd.as_ref().unwrap();
    let phone = &form.phone;
    let email = &form.email;
    let now = Local::now();

    let mut po = match req.extensions_mut().get_mut::<BeUser>() {
        Some(v) => v.clone(),
        None => {
            return returndata::fail("not find user");
        }
    };

    //检查 旧密码
    let old_passwd_calc = utils::md5_with_salt(old_passwd.as_str(), po.salt.as_str());
    if !old_passwd_calc.eq_ignore_ascii_case(po.password.as_str()) {
        // 就密码不正确
        return returndata::fail_msg("旧密码错误", "invalid oldPasswd");
    }

    let new_passwd_calc = utils::md5_with_salt(new_passwd.as_str(), po.salt.as_str());

    po.password = new_passwd_calc.clone();
    po.gmt_modified = now;
    po.phone = phone.clone();
    po.email = email.clone();
    po.name = Some(username.clone());

    //刷新token
    let new_token = Uuid::new_v4().to_string();
    po.token = Some(new_token);

    // 更新记录
    let ctx = app_state.ctx.clone();
    let affect = web::block(move || {
        ctx.web_dao.update_beuser_for_modify(&po)
    }).await;

    if let Err(e) = affect {
        error!("error, update_beuser_for_modify, {:?}", e);
        return returndata::fail("更新失败");
    }
    let affect = affect.unwrap();
    if affect != 1 {
        error!("error, update_beuser_for_modify, affect: {}", affect);
        return returndata::fail("更新失败");
    }

    returndata::success("succ".to_string())
}