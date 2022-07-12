use std::fmt::{self, Display, Debug};
use std::future::{ready, Ready};
use actix_web::{Responder, ResponseError, HttpRequest, HttpResponse, Result};
use actix_multipart::MultipartError;
// use actix_http::{Response, Error};
use serde::{Serialize};

pub const STATUS_OK: i32 = 0;
pub const STATUS_ERR_COMMON_FAIL: i32 = 1;
pub const STATUS_ERR_LOGINFAIL: i32 = 2;
pub const STATUS_ERR_SYSTEMERROR: i32 = 500;

pub const STATUS_ERR_UN_AUTHC: i32 = 101;
pub const STATUS_ERR_UN_AUTHZ: i32 = 102;

pub const STATUS_ERR_PARA_MISS: i32 = 201;
// 业务参数缺少
pub const STATUS_ERR_INVALID: i32 = 202; // 业务参数无效(格式或内容)或不存在

pub const MESSAGE_SUCCESS: &str = "操作成功";
pub const MESSAGE_COMMON_FAIL: &str = "操作失败";
pub const MESSAGE_ERR_UN_AUTHC: &str = "未登陆,请退出,重新登陆";

#[derive(Serialize)]
pub struct ReturnData<T>
    where T: serde::Serialize,
{
    pub status: i32,
    pub message: String,
    pub result: T,
}

impl<T: Serialize> Responder for ReturnData<T> {
    type Error = actix_web::Error;
    type Future = Ready<Result<HttpResponse, Self::Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .body(body)))
    }
}

//--------------------------------
/**
接口运行，有参数不合格，业务逻辑运行返回错误等，返回该对象
接口返回,无论操作成功和失败，http status code都为200，
通过 ReturnData.status !=0 来判断接口是否成功执行。
*/

pub type ReturnDataError = ReturnData<String>;

impl ReturnDataError {
    pub fn new(e: &str) -> Self {
        ReturnDataError {
            status: STATUS_ERR_COMMON_FAIL,
            message: MESSAGE_COMMON_FAIL.to_string(),
            result: e.to_string(),
        }
    }

    pub fn unauth(msg: &str) -> Self {
        ReturnDataError {
            status: STATUS_ERR_UN_AUTHC,
            message: MESSAGE_ERR_UN_AUTHC.to_string(),
            result: msg.to_string(),
        }
    }
}

impl From<actix_web::Error> for ReturnDataError {
    fn from(e: actix_web::Error) -> Self {
        ReturnDataError::new(&e.to_string())
    }
}

impl From<MultipartError> for ReturnDataError {
    fn from(e: MultipartError) -> Self {
        ReturnDataError::new(&e.to_string())
    }
}

impl Display for ReturnDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r#"{{"status":{},"message":"{}","result":"{}"}}"#, self.status, self.message, self.result)
    }
}

impl Debug for ReturnDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r#"{{"status":{},"message":"{}","result":"{}"}}"#, self.status, self.message, self.result)
    }
}

impl ResponseError for ReturnDataError {
    fn error_response(&self) -> HttpResponse {
        let body = serde_json::to_string(&self).unwrap();
        HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .body(body)
    }
}

impl From<String> for ReturnDataError {
    fn from(e: String) -> Self {
        ReturnDataError::new(&e)
    }
}

impl From<std::io::Error> for ReturnDataError {
    fn from(e: std::io::Error) -> Self {
        ReturnDataError::new(&e.to_string())
    }
}

pub type ReturnDataType<T> = std::result::Result<ReturnData<T>, ReturnDataError>;


pub fn success<T: Serialize>(result: T) -> ReturnDataType<T> {
    Ok(ReturnData {
        status: STATUS_OK,
        message: MESSAGE_SUCCESS.to_string(),
        result,
    })
}

pub fn success_str(result: &str) -> ReturnDataType<String> {
    Ok(ReturnData {
        status: STATUS_OK,
        message: MESSAGE_SUCCESS.to_string(),
        result: result.to_string(),
    })
}


pub fn fail<T: Serialize>(result: &str) -> ReturnDataType<T> {
    Err(ReturnDataError {
        status: STATUS_ERR_COMMON_FAIL,
        message: MESSAGE_COMMON_FAIL.to_string(),
        result: result.to_string(),
    })
}

pub fn fail_msg<T: Serialize>(msg: &str, result: &str) -> ReturnDataType<T> {
    Err(ReturnDataError {
        status: STATUS_ERR_COMMON_FAIL,
        message: msg.to_string(),
        result: result.to_string(),
    })
}
