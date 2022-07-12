use std::ffi::OsString;
use std::path::PathBuf;

use chrono::prelude::*;
use url::Url;
use crate::util::utils;

// -------------  facetrack -----------------------
pub fn get_facetrack_imgdir(root: &str, uuid: &str) -> OsString {
    let mut path = PathBuf::from(root);
    path.push("facetrack");
    if uuid.len() > 4 {
        path.push(&uuid[0..4]);
    } else {
        path.push(uuid);
    }
    path.push(uuid);
    path.into_os_string()
}


pub fn get_facetrack_full_bgpath(root: &str, uuid: &str) -> OsString {
    let mut path = PathBuf::from(get_facetrack_imgdir(root, uuid));
    path.push(format!("{}_bg.jpg", uuid));
    path.into_os_string()
}

pub fn get_facetrack_small_imgpath(root: &str, uuid: &str, face_id: i64) -> OsString {
    let mut path = PathBuf::from(get_facetrack_imgdir(root, uuid));
    path.push(format!("{}_{}_S.jpg", uuid, face_id));
    path.into_os_string()
}

pub fn get_facetrack_large_imgpath(root: &str, uuid: &str, face_id: i64) -> OsString {
    let mut path = PathBuf::from(get_facetrack_imgdir(root, uuid));
    path.push(format!("{}_{}_L.jpg", uuid, face_id));
    path.into_os_string()
}

pub fn get_facetrack_bgimg_url(prefix: &str, uuid: &str) -> String {
    format!("{}?cat=0&type=bg&id={}", prefix, uuid)
}

pub fn get_facetrack_smallimg_url(prefix: &str, uuid: &str, face_id: i64) -> String {
    format!("{}?cat=0&type=s&id={}&subid={}", prefix, uuid, face_id)
}

pub fn get_facetrack_largeimg_url(prefix: &str, uuid: &str, face_id: i64) -> String {
    format!("{}?cat=0&type=l&id={}&subid={}", prefix, uuid, face_id)
}

// -------------  person -----------------------
pub fn get_person_imgdir(root: &str, uuid: &str) -> OsString {
    let mut path = PathBuf::from(root);
    path.push("person");
    if uuid.len() > 4 {
        path.push(&uuid[0..4]);
    } else {
        path.push(uuid);
    }
    path.push(uuid);
    path.into_os_string()
}

pub fn get_person_full_imgpath(root: &str, uuid: &str, face_id: i64) -> OsString {
    let mut path = PathBuf::from(get_person_imgdir(root, uuid));
    path.push(format!("{}_{}.jpg", uuid, face_id));
    path.into_os_string()
}

pub fn get_person_full_coverpath(root: &str, uuid: &str) -> OsString {
    let mut path = PathBuf::from(get_person_imgdir(root, uuid));
    path.push(format!("{}_c.jpg", uuid));
    path.into_os_string()
}

pub fn get_person_img_url(prefix: &str, uuid: &str, face_id: i64) -> String {
    format!("{}?cat=1&type=s&id={}&subid={}", prefix, uuid, face_id)
}

pub fn get_person_cover_url(prefix: &str, uuid: &str) -> String {
    format!("{}?cat=1&type=c&id={}", prefix, uuid)
}

// -------------  cartrack -----------------------
pub fn get_cartrack_imgdir(root: &str, uuid: &str) -> OsString {
    let mut path = PathBuf::from(root);
    path.push("cartrack");
    if uuid.len() > 4 {
        path.push(&uuid[0..4]);
    } else {
        path.push(uuid);
    }
    path.push(uuid);
    path.into_os_string()
}

pub fn get_cartrack_full_bgpath(root: &str, uuid: &str) -> OsString {
    let mut path = PathBuf::from(get_cartrack_imgdir(root, uuid));
    path.push(format!("{}_bg.jpg", uuid));
    path.into_os_string()
}

pub fn get_cartrack_full_imgpath(root: &str, uuid: &str, face_id: i64) -> OsString {
    let mut path = PathBuf::from(get_cartrack_imgdir(root, uuid));
    path.push(format!("{}_{}_S.jpg", uuid, face_id));
    path.into_os_string()
}

pub fn get_caretrack_full_platepath(root: &str, uuid: &str) -> OsString {
    let mut path = PathBuf::from(get_cartrack_imgdir(root, uuid));
    path.push(format!("{}_p.jpg", uuid));
    path.into_os_string()
}

pub fn get_caretrack_full_platebinary_path(root: &str, uuid: &str) -> OsString {
    let mut path = PathBuf::from(get_cartrack_imgdir(root, uuid));
    path.push(format!("{}_bin.jpg", uuid));
    path.into_os_string()
}


pub fn get_cartrack_bgimg_url(prefix: &str, uuid: &str) -> String {
    format!("{}?cat=2&type=bg&id={}", prefix, uuid)
}

pub fn get_cartrack_img_url(prefix: &str, uuid: &str, face_id: i64) -> String {
    format!("{}?cat=2&type=s&id={}&subid={}", prefix, uuid, face_id)
}

pub fn get_cartrack_plate_url(prefix: &str, uuid: &str) -> String {
    format!("{}?cat=2&type=p&id={}", prefix, uuid)
}

pub fn get_cartrack_platebinary_url(prefix: &str, uuid: &str) -> String {
    format!("{}?cat=2&type=bin&id={}", prefix, uuid)
}

// -------------  util -----------------------
pub fn get_item_from_idscores(ids: &str) -> std::result::Result<Vec<(i64, f64)>, String> {
    let items: Vec<&str> = ids.split(',').collect();

    let mut list = Vec::new();
    for v in items {
        let pair: Vec<&str> = v.split(':').collect();
        if pair.len() != 2 {
            return Err(format!("{} is invalid", v));
        }
        let id = match pair.get(0).unwrap().parse::<i64>() {
            Ok(v) => v,
            Err(e) => {
                return Err(format!("{}", e));
            }
        };

        let score = match pair.get(1).unwrap().parse::<f64>() {
            Ok(v) => v,
            Err(e) => {
                return Err(format!("{}", e));
            }
        };
        list.push((id, score));
    }
    Ok(list)
}

pub fn get_debug_rtsp(ip: &str, port: i64, src_id: &str) -> String {
    format!("rtsp://{}:{}/{}", ip, port, src_id)
}

pub fn get_ip_from_rtsp(rtsp_url: &str, defalut_value: &str) -> String {
    let rst = Url::parse(rtsp_url);
    if let Ok(v) = rst {
        if let Some(ip) = v.host_str() {
            return ip.to_string();
        }
    }
    defalut_value.to_string()
}

/// /module(upload)/yyyy/mm/dd
pub fn get_upload_relate_path(module: &str, now: DateTime<Local>) -> String {
    let ts = now.format("%Y/%m/%d");
    format!("/{}/{}", module, ts.to_string())
}

/// /root + module(upload)/yyyy/mm/dd
pub fn get_upload_full_path(root: &str, relate_path: &str) -> String {
    let mut path = format!("{}/{}", root, relate_path);
    while path.contains("//") {
        path = path.replace("//", "/")
    }
    path
}

/// /root/module(upload)/yyyy/mm/dd/ + xxxxxxxxx.jpg
pub fn get_upload_img_path(dir: &str, file_name: &str) -> OsString {
    let mut path = PathBuf::from(dir);
    path.push(file_name);
    path.into_os_string()
}

/// /root + /module(upload)/yyyy/mm/dd/xxxxxxxxx.jpg
pub fn get_upload_imgpath_by_relate(root: &str, relate_img_path: &str) -> OsString {
    let path = format!("{}/{}", root, relate_img_path);
    let path = utils::clean_relate_path(&path);

    path.into()
}