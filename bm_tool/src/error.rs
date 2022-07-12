#![allow(dead_code)]

use std::fmt::Debug;
use tokio::task::JoinError;

#[derive(Debug)]
pub struct AppError {
    pub msg: String
}

pub type AppResult<T> = std::result::Result<T, AppError>;

impl AppError {
    pub fn new(s: &str) -> Self {
        AppError {
            msg: s.to_string()
        }
    }

    pub fn from_debug<T>(t: T) -> Self
        where T: Debug {
        AppError { msg: format!("{:?}", t) }
    }
}


impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError {
            msg: format!("{}", e),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError {
            msg: format!("{}", e),
        }
    }
}

impl From<tokio::task::JoinError> for AppError {
    fn from(e: JoinError) -> Self {
        AppError {
            msg: format!("{}", e),
        }
    }
}

impl From<cffc_base::db::dbop::Error> for AppError {
    fn from(e: cffc_base::db::dbop::Error) -> Self {
        AppError {
            msg: format!("{:?}", e),
        }
    }
}


impl From<cffc_base::api::bm_api::ApiError> for AppError {
    fn from(e: cffc_base::api::bm_api::ApiError) -> Self {
        AppError {
            msg: format!("{:?}", e),
        }
    }
}