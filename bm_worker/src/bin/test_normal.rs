#![allow(dead_code)]
#![allow(unused_imports)]

use cffc_base::util::{self, utils};
use std::path::{PathBuf};
use cffc_base::model::img_file;
use chrono::prelude::*;
use cffc_base::api::lane::{Lanes, LaneLine, LanePoint};

fn test_lane() {
    let lanes = Lanes {
        lines: vec![LaneLine {
            top: LanePoint {
                x: 716,
                y: 430,
            },
            btm: LanePoint {
                x: 189,
                y: 1050,
            },
        }, LaneLine {
            top: LanePoint {
                x: 886,
                y: 452,
            },
            btm: LanePoint {
                x: 706,
                y: 1050,
            },
        }, LaneLine {
            top: LanePoint {
                x: 1086,
                y: 434,
            },
            btm: LanePoint {
                x: 1210,
                y: 1050,
            },
        }, LaneLine {
            top: LanePoint {
                x: 1279,
                y: 436,
            },
            btm: LanePoint {
                x: 1678,
                y: 1050,
            },
        }, LaneLine {
            top: LanePoint {
                x: 1449,
                y: 436,
            },
            btm: LanePoint {
                x: 1911,
                y: 882,
            },
        }]
    };

    let json_str = serde_json::to_string(&lanes).unwrap();
    println!("{}", json_str);


    let des = Lanes::fromstr(json_str.as_str());
    println!("{:?}", des);


    let point = LanePoint {
        x: 750,
        y: 430,
    };

    let index = lanes.get_lane_num(&point);
    println!("index:{}", index);

    let index = lanes.get_vehicle_lane_num(&point, true);
    println!("index:{}", index);

    let index = lanes.get_vehicle_lane_num(&point, false);
    println!("index:{}", index);
}


fn test_md5() {
    let s = "123";
    println!("->{}", util::utils::md5_it(s));
}

fn test_path_1() {
    let mut p = PathBuf::from("../data/bak//");
    p.push("2020/12/13");
    p.push("abc");

    println!("{:?}", p.into_os_string());
}

fn test_path() {
    let path = utils::get_file_extension("/abc/def/aaa");
    println!("{:?}", path);
}

fn test_chrono() {
    println!("haha");
    let s = "2020-12-17 11:08:06";
    let rst = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S");
    println!("{:?}", rst);
    let rst = rst.unwrap();
    let ll = Local;

    let d = ll.from_local_datetime(&rst);
    println!("{:?}", d);
}

fn test_urlpath() {
    let s = "/ws/track/";
    let a: Vec<&str> = s.split("/").collect();
    if a.len() >= 3 {
        println!("{}", a.get(2).unwrap());
    }

    println!("{:?}", a);
}

fn test_clean() {
    let s = "晋S  T7V53";
    let opt_s = Some(s.to_string());
    let opt_s2 = utils::clean_space_option_string(&opt_s);
    println!("{:?}", opt_s2);
}

fn main() {
    // test_chrono();
    // test_lane();
    // test_urlpath();
    test_clean();
}