#![allow(unused_imports)]
use tokio::fs;
use image::ImageFormat;
use std::io::Cursor;
use image::io::Reader;

pub fn check_bmp_magic(buf: &[u8]) -> bool {
    if buf.len() >= 2 && buf[0] == 0x42 && buf[1] == 0x4d {
        true
    } else {
        false
    }
}

#[tokio::main]
async fn main() {
    println!("test bmp");

    let file_name = "/Users/tom/Desktop/a.jpg";
    let dst_name = "/Users/tom/Desktop/a_c.jpg";

    let content = fs::read(&file_name).await;
    if let Err(e) = content {
        println!("{:?}", e);
        return;
    }
    let content = content.unwrap();
    println!("{:02X} {:02X}", content[0], content[1]);

    println!("check: {}", check_bmp_magic(&content));

    let reader = Reader::new(Cursor::new(content)).with_guessed_format();
    if let Err(e) = reader {
        println!("{:?}", e);
        return;
    }
    let reader = reader.unwrap();
    println!("format: {:?}", reader.format());

    let img = reader.decode().unwrap();

    let mut bytes: Vec<u8> = Vec::new();
    let converted = img.write_to(&mut bytes, image::ImageOutputFormat::Jpeg(85));
    if let Err(e) = converted {
        println!("{:?}", e);
        return;
    }
    let _saved = fs::write(&dst_name, bytes).await;
    if let Err(e) = converted {
        println!("{:?}", e);
        return;
    }
    println!("saved");
}