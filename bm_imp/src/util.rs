use std::ffi::OsString;
use std::path::{Path, PathBuf};

use tokio::fs::{self, DirBuilder};

pub async fn read_file_base64(path: impl AsRef<Path>) -> std::io::Result<String> {
    let content = fs::read(path).await?;
    Ok(base64::encode(content))
}


fn io_error(msg: &str) -> std::io::Error {
    use std::io::{Error, ErrorKind};
    Error::new(ErrorKind::Other, msg)
}

pub async fn write_file_base64(path: impl AsRef<Path>, base64_content: &str) -> std::io::Result<()> {
    let path = path.as_ref().to_owned();

    let dir = match path.parent() {
        Some(v) => v,
        None => {
            return Err(io_error("no parent"));
        }
    };

    // 建立目录
    let _ = DirBuilder::new().recursive(true).create(dir).await?;

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

pub fn get_full_imgpath(root: &str, person_id: &str, face_id: i64) -> OsString {

    // /extdata/df_imgs/person/0037/003794dc-8283-4c1d-a262-661ed60c009b/003794dc-8283-4c1d-a262-661ed60c009b_341019.jpg
    let mut path = PathBuf::from(root);
    path.push("person");
    if person_id.len() > 4 {
        path.push(&person_id[..4]);
    }

    path.push(person_id);
    path.push(format!("{}_{}.jpg", person_id, face_id));

    path.into_os_string()
}