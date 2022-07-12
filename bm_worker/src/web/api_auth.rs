#![allow(clippy::type_complexity)]

use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::{Error, HttpMessage, HttpResponse, web};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::error::QueryPayloadError;
use actix_web::web::Query;
use futures::future::{Either, ok, Ready};
use log::{debug, error};
use serde::Deserialize;

use cffc_base::model::returndata::ReturnDataError;
use cffc_base::util::utils;

use crate::error::{AppError, AppResult};
use crate::web::AppState;

pub struct ApiAuthFilter {
    pub prefix: String,
}

pub struct ApiAuthMiddleware<S> {
    service: S,
    prefix: String,
}

impl ApiAuthFilter {
    pub fn new(prefix: &str) -> Self {
        ApiAuthFilter {
            prefix: prefix.to_string(),
        }
    }
}

impl<S> ApiAuthMiddleware<S> {
    pub fn new(s: S, prefix: &str) -> ApiAuthMiddleware<S> {
        ApiAuthMiddleware {
            service: s,
            prefix: prefix.to_string(),
        }
    }
}


impl<S, B> Transform<S> for ApiAuthFilter
    where S: Service<Request=ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
          S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = ApiAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        // debug!("ApiAuthFilter, new_transform");
        ok(ApiAuthMiddleware::new(service, self.prefix.as_str()))
    }
}


impl<S, B> Service for ApiAuthMiddleware<S>
    where S: Service<Request=ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
          S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // debug!("ApiAuthFilter, poll_ready");
        self.service.poll_ready(ctx)
    }

    fn call(&mut self, req: Self::Request) -> Self::Future {
        // debug!("ApiAuthFilter, call");

        // 如果不是过滤路径，直接放过
        if !req.path().starts_with(self.prefix.as_str()) {
            return Either::Left(self.service.call(req));
        }

        let is_auth = check_query_digest(&req);
        if let Err(e) = is_auth {
            error!("error, ApiAuthFilter, check_query_digest: {:?}", e);
            let data = ReturnDataError::unauth("check digest error");
            return Either::Right(ok(req.into_response(HttpResponse::Ok().json(data).into_body())));
        }
        let is_auth = is_auth.unwrap();

        if is_auth {
            Either::Left(self.service.call(req))
        } else {
            let data = ReturnDataError::unauth("auth fail");
            Either::Right(ok(req.into_response(HttpResponse::Ok().json(data).into_body())))
        }
    }
}

/// http://xxx/api/xxx?uid=abc&ts=1234&sign=45678
// sign=md5(uid+ts+token)
#[derive(Deserialize)]
struct ApiDigestData {
    uid: String,
    ts: String,
    sign: String,
}

impl ApiDigestData {
    fn validate(&self) -> bool {
        if self.sign.is_empty() || self.ts.is_empty() || self.sign.is_empty() {
            return false;
        }
        true
    }
}

fn check_query_digest(req: &ServiceRequest) -> AppResult<bool> {
    let app_state: Option<&web::Data<AppState>> = req.app_data();

    let query: Result<Query<ApiDigestData>, QueryPayloadError> = Query::from_query(req.query_string());
    if let Err(e) = query {
        return Err(AppError::from_debug(e));
    }
    let query = query.unwrap();
    debug!("ApiAuthFilter, [uid:{}, ts:{}, sign:{}]", query.uid, query.ts, query.sign);

    if !query.0.validate() {
        return Err(AppError::new("ApiDigestData isn't validated"));
    }

    if app_state.is_none() {
        return Err(AppError::new("can't find AppState"));
    }
    let app_state = app_state.unwrap();

    let ctx = app_state.ctx.clone();
    let login_name = query.uid.clone();

    let po = ctx.web_dao.load_beuser_by_loginname(&login_name);

    if let Err(e) = po {
        return Err(AppError::from(e));
    }
    let po = po.unwrap();
    if po.is_none() {
        // 没有这个用户
        return Ok(false);
    }
    let po = po.unwrap();
    //userName + ts + po.Token
    let sign_calc = utils::md5_it(&format!("{}{}{}",
                                           query.uid, query.ts, po.token.as_ref().map_or("", |x| x.as_str())));

    debug!("ApiAuthFilter, sign:{}, calc:{}", query.sign, sign_calc);

    if sign_calc.eq_ignore_ascii_case(query.sign.as_str()) {
        // po 放入 内存中，供后续使用
        req.extensions_mut().insert(po);
        Ok(true)
    } else {
        Ok(false)
    }
}

