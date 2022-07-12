use actix_multipart::Multipart;
use actix_web::web;
use chrono::prelude::*;
use futures::StreamExt;
use log::{debug, error, info};
use serde_json::{self, Result as JsonResult};

use cffc_base::api::bm_api::{CarNotifyParams, FaceNotifyParams};
use cffc_base::model::returndata::{self, ReturnDataType};
use cffc_base::util::multipart_form::{self, MultipartFormValues};

use crate::queue_item::{NotifyCarQueueItem, NotifyFaceQueueItem};
use crate::web::AppState;

fn print_time_use(ts_start: DateTime<Local>) {
    let ts_use = Local::now().signed_duration_since(ts_start).num_milliseconds();
    debug!("track_upload end, use: {} ms", ts_use);
}

pub async fn test_upload(_data: web::Data<AppState>, mut payload: web::Payload) -> ReturnDataType<String> {
    info!("test_upload begin ...");
    // let values = multipart_form::parse_multi_form(multi_payload).await;
    // info!("test_upload, after parse_multi_form");
    //
    // info!("test_upload, values: {:?}", values);

    let mut body = web::BytesMut::new();
    loop {
        let chunk = payload.next().await;
        if chunk.is_none() {
            info!("test_upload, chunk is none");
            break;
        }
        let chunk = chunk.unwrap();
        if let Err(e) = chunk {
            error!("error, test_upload, {:?}", e);
            return returndata::fail("fail");
        }

        let chunk = chunk.unwrap();
        body.extend_from_slice(&chunk);
    }

    // while let Some(chunk) = payload.next().await {
    //     if let Err(e) = chunk {
    //         error!("error, test_upload, {:?}", e);
    //         return returndata::fail("fail");
    //     }
    //
    //     let chunk = chunk.unwrap();
    //     body.extend_from_slice(&chunk);
    // }
    info!("test_upload, payload.len:{}", body.len());


    returndata::success_str("ok")
}

pub async fn track_upload(data: web::Data<AppState>, payload: Multipart) -> ReturnDataType<String> {
    let ts_start = Local::now();
    debug!("track_upload begin ...");

    let values = multipart_form::parse_multi_form(payload).await;
    debug!("track_upload, after parse_multi_form");
    if let Err(e) = values {
        error!("error, parse_multi_form, {}", e);

        print_time_use(ts_start);
        return returndata::fail(e.as_str());
    }

    let values = values.unwrap();
    // info!("track_upload, {:?}", values);

    if let Some(notify_type) = values.get_string_value("type") {
        match notify_type.as_str() {
            "facetrack" => {
                let rst = handle_face(data, values).await;
                print_time_use(ts_start);
                return rst;
            }
            "vehicletrack" => {
                let rst = handle_car(data, values).await;
                print_time_use(ts_start);
                return rst;
            }
            _ => {
                error!("error, unknown type, {}", notify_type);
                print_time_use(ts_start);
                return returndata::fail(&format!("unknown type, {}", notify_type));
            }
        }
    } else {
        error!("error, track_upload, param: type not found");
        error!("error, no type, values: {:?}", values);
        print_time_use(ts_start);
        return returndata::fail("param: type not found");
    }
}

async fn handle_face(data: web::Data<AppState>, values: MultipartFormValues) -> ReturnDataType<String> {
    let now = Local::now();
    let json_str = match values.get_string_value("json") {
        Some(v) => v,
        None => {
            error!("error, param: json not found");
            error!("error, no json, values: {:?}", values);

            return returndata::fail("param: json not found");
        }
    };

    debug!("->face:{}", json_str);

    let notify: JsonResult<FaceNotifyParams> = serde_json::from_reader(json_str.as_bytes());
    if let Ok(mut item) = notify {
        info!("recv track, {}, index:{}, ft", item.id, item.index);

        // 处理图片
        item.background.image_buf =
            match values.get_file_value(item.background.image_file.as_str()) {
                Some((_, v)) => v,
                None => {
                    error!("error, can't find para: {}", item.background.image_file);
                    return returndata::fail(format!("can't find para: {}", item.background.image_file).as_str());
                }
            };

        for x in item.faces.iter_mut() {
            x.aligned_buf = match values.get_file_value(x.aligned_file.as_str()) {
                Some((_, v)) => v,
                None => {
                    error!("error, can't find para: {}", x.aligned_file);
                    return returndata::fail(format!("can't find para: {}", x.aligned_file).as_str());
                }
            };

            x.display_buf = match values.get_file_value(x.display_file.as_str()) {
                Some((_, v)) => v,
                None => {
                    error!("error, can't find para: {}", x.display_file);
                    return returndata::fail(format!("can't find para: {}", x.display_file).as_str());
                }
            };

            if let Some(ref feature_file) = x.feature_file {
                if !feature_file.is_empty() {
                    x.feature_buf = match values.get_file_value(feature_file.as_str()) {
                        Some((_, v)) => Some(v),
                        None => {
                            error!("error, can't find para: {}", feature_file);
                            return returndata::fail(format!("can't find para: {}", feature_file).as_str());
                        }
                    }
                } else {
                    x.feature_file = None;
                    debug!("{}, has no feature", item.id);
                }
            } else {
                x.feature_file = None;
                debug!("{}, has no feature", item.id);
            }
        }
        data.face_queue.push(NotifyFaceQueueItem {
            uuid: item.id.clone(),
            notify: item,
            ts: now,
        });
        debug!("track_upload, end push face");
        returndata::success_str("ok")
    } else {
        error!("error, {:?}", notify);
        returndata::fail("json parse fail")
    }
}

async fn handle_car(data: web::Data<AppState>, values: MultipartFormValues) -> ReturnDataType<String> {
    let now = Local::now();

    let json_str = match values.get_string_value("json") {
        Some(v) => v,
        None => {
            error!("error, param: json not found");
            error!("error, no json, values: {:?}", values);
            return returndata::fail("param: json not found");
        }
    };
    debug!("->car:{}", json_str);

    let notify: JsonResult<CarNotifyParams> = serde_json::from_reader(json_str.as_bytes());
    if let Ok(mut item) = notify {
        info!("recv track, {}, index:{}, ct", item.id, item.index);

        // 处理图片
        item.background.image_buf =
            match values.get_file_value(item.background.image_file.as_str()) {
                Some((_, v)) => v,
                None => {
                    error!("error, can't find para: {}", item.background.image_file);
                    return returndata::fail(format!("can't find para: {}", item.background.image_file).as_str());
                }
            };

        for x in item.vehicles.iter_mut() {
            x.img_buf = match values.get_file_value(x.image_file.as_str()) {
                Some((_, v)) => v,
                None => {
                    error!("error, can't find para: {}", x.image_file);
                    return returndata::fail(format!("can't find para: {}", x.image_file).as_str());
                }
            };
        }

        // 有牌照号码
        if item.has_plate_info() {
            let x = item.plate_info.as_mut().unwrap();
            if let Some(ref img) = x.image_file {
                x.img_buf = match values.get_file_value(img.as_str()) {
                    Some((_, v)) => v,
                    None => {
                        error!("error, can't find para: {}", img);
                        return returndata::fail(format!("can't find para: {}", img).as_str());
                    }
                }
            } else {
                error!("error, has plate text, but hasn't plate img");
            }
        }

        if item.has_plate_binary() {
            let x = item.plate_info.as_mut().unwrap();
            if let Some(ref img) = x.binary_file {
                x.binary_buf = match values.get_file_value(img.as_str()) {
                    Some((_, v)) => v,
                    None => {
                        error!("error, can't find para: {}", img);
                        return returndata::fail(format!("can't find para: {}", img).as_str());
                    }
                }
            } else {
                error!("error, has plate binary, but hasn't plate binary img");
            }
        }


        debug!("track_upload, will push car");
        data.car_queue.push(NotifyCarQueueItem {
            uuid: item.id.clone(),
            notify: item,
            ts: now,
        });
        debug!("track_upload, end push car");
        returndata::success_str("ok")
    } else {
        error!("error, {:?}", notify);
        returndata::fail("json parse fail")
    }
}
