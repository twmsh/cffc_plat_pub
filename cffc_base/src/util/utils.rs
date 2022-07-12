use std::io::Cursor;
use std::path::Path;

use chrono::LocalResult;
use chrono::prelude::*;
use crypto::digest::Digest;
use crypto::md5::Md5;
use deadqueue::unlimited::Queue;
use serde::{Deserialize, Serialize};
use tokio::fs::{self, DirBuilder};

//-------------------- const --------------------------
pub const DATETIME_FMT_SHORT: &str = "%Y-%m-%d %H:%M:%S";

//-------------------- struct --------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DateRange {
    pub begin: DateTime<Local>,
    pub end: DateTime<Local>,
}

impl DateRange {
    pub fn from_str(begin_nv: &str, end_nv: &str, fmt: &str) -> Option<Self> {
        let begin = parse_localtime_str(begin_nv, fmt);
        if begin.is_err() {
            return None;
        }
        let begin = begin.unwrap();

        let end = parse_localtime_str(end_nv, fmt);
        if end.is_err() {
            return None;
        }
        let end = end.unwrap();

        Some(DateRange {
            begin,
            end,
        })
    }

    pub fn from_option_str(begin_nv: &Option<String>, end_nv: &Option<String>, fmt: &str) -> Option<Self> {
        if begin_nv.is_none() || end_nv.is_none() {
            return None;
        }

        let begin_str = begin_nv.as_ref().unwrap();
        let end_str = end_nv.as_ref().unwrap();

        Self::from_str(begin_str, end_str, fmt)
    }
}

pub fn parse_localtime_str(ts: &str, fmt: &str) -> std::result::Result<DateTime<Local>, String> {
    let dt = NaiveDateTime::parse_from_str(ts, fmt);
    if let Err(e) = dt {
        return Err(format!("{}", e));
    }
    let dt = dt.unwrap();
    let rst = (Local).from_local_datetime(&dt);
    if let LocalResult::Single(v) = rst {
        Ok(v)
    } else {
        Err(format!("invalid {:?}", rst))
    }
}


//-------------------- util --------------------------
pub async fn prepare_dir(path: impl AsRef<Path>) -> std::io::Result<()> {
    DirBuilder::new().recursive(true).create(path).await
}

pub async fn remove_dir(dir: impl AsRef<Path>) -> std::io::Result<()> {
    tokio::fs::remove_dir_all(dir).await
}

pub async fn remove_file(file: impl AsRef<Path>) -> std::io::Result<()> {
    tokio::fs::remove_file(file).await
}

pub async fn copy_file(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<u64> {
    fs::copy(src, dst).await
}

fn io_error(msg: &str) -> std::io::Error {
    use std::io::{Error, ErrorKind};
    Error::new(ErrorKind::Other, msg)
}

pub async fn read_file_base64(path: impl AsRef<Path>) -> std::io::Result<String> {
    let content = fs::read(path).await?;
    Ok(base64::encode(content))
}


pub async fn write_file_base64(path: impl AsRef<Path>, base64_content: &str) -> std::io::Result<()> {
    let path = path.as_ref().to_owned();

    // 解码base64
    let buf = match base64::decode(base64_content) {
        Ok(v) => v,
        Err(_) => {
            return Err(io_error("base64 decoe fail"));
        }
    };

    // 保存文件
    fs::write(path, buf).await
}

pub async fn write_file(path: impl AsRef<Path>, content: &[u8]) -> std::io::Result<()> {
    fs::write(path, content).await
}

pub fn get_disk_available(path: impl AsRef<Path>) -> std::io::Result<u64> {
    fs2::available_space(path)
}


/// 队列中数据时，一次最多取出limit数据, 没有数据时候阻塞，取一条数据返回
pub async fn pop_queue_batch<T>(queue: &Queue<T>, max: usize) -> Vec<T> {
    let mut size = 0_usize;
    let mut list = Vec::new();

    while let Some(v) = queue.try_pop() {
        list.push(v);
        size += 1;
        if size == max {
            break;
        }
    }

    if list.is_empty() {
        let v = queue.pop().await;
        list.push(v);
    }
    list
}

pub fn md5_it(s: &str) -> String {
    let mut md5 = Md5::new();
    md5.input_str(s);
    md5.result_str()
}

pub fn md5_with_salt(s: &str, salt: &str) -> String {
    let mut md5 = Md5::new();
    md5.input_str(s);
    md5.input_str(salt);
    md5.result_str()
}

pub fn get_file_extension(path: &str) -> Option<String> {
    let p = Path::new(path).extension();
    if let Some(v) = p {
        if let Some(vv) = v.to_str() {
            return Some(vv.to_string());
        }
    }
    None
}


//----------- web utils --------------
pub fn must_length(str: &str, min: usize, max: usize) -> bool {
    let len = str.chars().count();

    if len >= min && len <= max {
        return true;
    }
    false
}

pub fn option_must_length(str: &Option<String>, min: usize, max: usize) -> bool {
    if let Some(ref v) = str {
        let len = v.chars().count();
        if len >= min && len <= max {
            return true;
        }
    }

    false
}

pub fn option_must_notempty(str: &Option<String>) -> bool {
    if let Some(ref v) = str {
        let len = v.trim().len();
        if len > 0 {
            return true;
        }
    }

    false
}

pub fn option_must_datetime(str: &Option<String>, fmt: &str) -> bool {
    if let Some(ref v) = str {
        let value = NaiveDateTime::parse_from_str(v.as_str(), fmt);
        return value.is_ok();
    }
    false
}

pub fn option_should_datetime(str: &Option<String>, fmt: &str) -> bool {
    if let Some(ref v) = str {
        let value = NaiveDateTime::parse_from_str(v.as_str(), fmt);
        return value.is_ok();
    }
    true
}


pub fn option_must_num(str: &Option<String>) -> bool {
    if let Some(ref v) = str {
        let value = v.parse::<i64>();
        return value.is_ok();
    }
    false
}

pub fn option_must_float(str: &Option<String>) -> bool {
    if let Some(ref v) = str {
        let value = v.parse::<f64>();
        return value.is_ok();
    }
    false
}


pub fn option_must_num_range(str: &Option<String>, min: i64, max: i64) -> bool {
    if let Some(ref v) = str {
        let value = v.parse::<i64>();
        if let Ok(num) = value {
            if num >= min && num <= max {
                return true;
            }
        }
    }
    false
}

/// 如果有值，则值为数字，且范围在 min <= x <= max
pub fn option_should_num_range(str: &Option<String>, min: i64, max: i64) -> bool {
    if str.is_none() {
        return true;
    }

    let str = str.as_ref().unwrap();
    // 字符串为空
    if str.is_empty() {
        return true;
    }

    let value = str.parse::<i64>();
    // 有值，但是不是数字
    if value.is_err() {
        return false;
    }
    let value = value.unwrap();
    if value >= min && value <= max {
        // 满足范围限定
        return true;
    }

    // 不满足范围限定
    false
}

/// str 必须为数字字符串
pub fn get_option_must_num(str: &Option<String>) -> i64 {
    str.as_ref().unwrap().parse().unwrap()
}

/// str 为空或数字字符串
/// 为非数字字符串时候,返回None
pub fn get_option_num(str: &Option<String>) -> Option<i64> {
    if let Some(v) = str {
        if let Ok(num) = v.parse::<i64>() {
            return Some(num);
        }
    }

    None
}

/// str 为空或数字字符串
/// 为非数字字符串时候,返回None
pub fn get_option_float(str: &Option<String>) -> Option<f64> {
    if let Some(v) = str {
        if let Ok(num) = v.parse::<f64>() {
            return Some(num);
        }
    }

    None
}

/// trim字符串，如果字符串为空，返回None
///
pub fn clean_option_string(str: &Option<String>) -> Option<String> {
    if let Some(x) = str {
        let value = x.trim();
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

/// trim字符串，删除内容中的空格，如果字符串为空，返回None
///
pub fn clean_space_option_string(str: &Option<String>) -> Option<String> {
    if let Some(x) = str {
        let value = x.trim().replace(" ", "").replace("　", "");
        if !value.is_empty() {
            return Some(value);
        }
    }
    None
}


pub fn unwarp_option_string(str: &Option<String>, default: &str) -> String {
    if let Some(x) = str {
        let value = x.trim();
        if !value.is_empty() {
            return value.to_string();
        }
    }

    default.to_string()
}

pub fn get_option_datetime(date: &Option<DateTime<Local>>, default: DateTime<Local>) -> DateTime<Local> {
    date.map_or_else(|| default, |x| x)
}

pub fn clean_relate_path(s: &str) -> String {
    let mut path = s.to_string();
    while path.contains("..") {
        path = path.replace("..", "")
    }

    while path.contains("//") {
        path = path.replace("//", "/")
    }

    path
}

//--------------
pub fn check_bmp_magic(buf: &[u8]) -> bool {
    buf.len() >= 2 && buf[0] == 0x42 && buf[1] == 0x4d
}

/// 检查content是否是bmp，如果是，需要转成jpg存储
pub async fn write_jpg_file(path: impl AsRef<Path>, content: &[u8]) -> std::io::Result<()> {
    if !check_bmp_magic(content) {
        return fs::write(path, content).await;
    }

    let reader = image::io::Reader::new(Cursor::new(content)).with_guessed_format()?;
    let img = reader.decode();
    if let Err(e) = img {
        let img_error = std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e));
        return Err(img_error);
    }
    let img = img.unwrap();

    let mut bytes: Vec<u8> = Vec::new();
    let converted = img.write_to(&mut bytes, image::ImageOutputFormat::Jpeg(85));
    if let Err(e) = converted {
        let img_error = std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e));
        return Err(img_error);
    }

    fs::write(path, bytes).await
}

