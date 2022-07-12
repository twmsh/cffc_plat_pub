#![allow(unused_parens)]

use std::time::Instant;

use actix::Addr;
use actix_files::Files;
use actix_web::{Error, http::Method, HttpRequest, HttpResponse, web};
use actix_web_actors::ws;
use log::debug;

use crate::services::ws::agent::WsAgent;
use crate::services::ws::session::WsSession;
use crate::web::controllers::{admin_ctl, coi_ctl};
use crate::web::controllers::camera_ctl;
use crate::web::controllers::cartrack_ctl;
use crate::web::controllers::crop_ctl;
use crate::web::controllers::facetrack_ctl;
use crate::web::controllers::getsingleimg;
use crate::web::controllers::home_ctl;
use crate::web::controllers::logon;
use crate::web::controllers::notify_handle;
use crate::web::controllers::poi_ctl;

async fn ws_route(web::Path((room)): web::Path<(String)>, req: HttpRequest,
                  stream: web::Payload, srv: web::Data<Addr<WsAgent>>) -> Result<HttpResponse, Error> {
    debug!("WS, ws_route, room: {}", room);

    ws::start(
        WsSession {
            id: 0,
            hb: Instant::now(),
            room: room.clone(),
            name: None,
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}


pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(Files::new("/css", "./static/css").prefer_utf8(true).disable_content_disposition())
        .service(Files::new("/imgs", "./static/imgs").disable_content_disposition())
        .service(Files::new("/js", "./static/js").prefer_utf8(true).disable_content_disposition())
        .service(Files::new("/upload", "./static/upload").disable_content_disposition())
        .service(web::resource("/ws/{room}").to(ws_route))
        .route("/testupload", web::post().to(notify_handle::test_upload))
        .route("/trackupload", web::post().to(notify_handle::track_upload))
        .route("/getsingleimg", web::get().to(getsingleimg::get))
        .route("/", web::get().to(logon::login))
        .route("/logon", web::post().to(logon::logon))
        .route("/main", web::get().to(logon::home))
        .service(web::scope("/api")
            .route("/logout", web::post().to(logon::logout))
            .route("/admin/detail", web::get().to(admin_ctl::detail))
            .route("/admin/modify", web::post().to(admin_ctl::modify))
            .route("/home/getDisplayCameras", web::get().to(home_ctl::get_display_cameras))
            .route("/home/getInitAlarmList", web::get().to(home_ctl::get_init_alarm_list))
            .route("/camera/list", web::get().to(camera_ctl::list))
            .route("/camera/add", web::post().to(camera_ctl::add))
            .route("/camera/delete", web::post().to(camera_ctl::delete))
            .route("/camera/modify", web::post().to(camera_ctl::modify))
            .route("/camera/setOnScreen", web::post().to(camera_ctl::set_on_screen))
            .route("/camera/setState", web::post().to(camera_ctl::set_state))

            .route("/group/list", web::get().to(poi_ctl::group_list))
            .route("/poi/detail", web::get().to(poi_ctl::detail))
            .route("/poi/list", web::get().to(poi_ctl::list))
            .route("/poi/add", web::post().to(poi_ctl::add))
            .route("/poi/delete", web::post().to(poi_ctl::delete))
            .route("/poi/modify", web::post().to(poi_ctl::modify))

            .route("/facetrack/list", web::get().to(facetrack_ctl::list))
            .route("/cartrack/list", web::get().to(cartrack_ctl::list))

            .route("/coi/group_list", web::get().to(coi_ctl::group_list))
            .route("/coi/detail", web::get().to(coi_ctl::detail))
            .route("/coi/list", web::get().to(coi_ctl::list))
            .route("/coi/add", web::post().to(coi_ctl::add))
            .route("/coi/delete", web::post().to(coi_ctl::delete))
            .route("/coi/modify", web::post().to(coi_ctl::modify))


            .service(
                web::resource("/crop")
                    .route(web::post().to(crop_ctl::crop))
                    .route(web::method(Method::OPTIONS).to(crop_ctl::do_options))
            )
        );
}