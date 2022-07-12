#![allow(dead_code)]
#![allow(unused_assignments)]

use serde_json::{Value};


fn main() {
    test_json_field();
}

fn test_json_field() {
    let s = r#"
 {
                "bg_width": 0,
                "cv_force_tpu": false,
                "draw_rect_on_bg": true,
                "enable_plate": true,
                "enable_plate_merge": true,
                "enable_portrait": true,
                "enable_vehicle_classify": true,
                "face_cluster_threshold": 0.60000002384185791,
                "face_duplicate_threshold": 0.89999997615814209,
                "face_mask": false,
                "face_max_feature": 3,
                "face_merge_threshold": 0.69999998807907104,
                "face_merge_window": 2000,
                "feature_count": 1,
                "max_plate_height": 0.30000001192092896,
                "max_targets_in_track": 3,
                "min_face_history_quality": 0.5,
                "min_plate_bit_score": 0.20000000298023224,
                "min_plate_width": 20,
                "min_vehicle_bit_score": 0.20000000298023224,
                "plate_binary": true,
                "plate_filter_rate": 0.30000001192092896,
                "plate_height": 0,
                "plate_precision": 2,
                "plate_width": 0,
                "remove_vehicle_without_plate": true,
                "size_filter_rate": 0.25,
                "stationary_iou_threshold": 0,
                "vehicle_history_max_duration": 10,
                "vehicle_history_max_size": 20,
                "vehicle_precision": 2
            }
"#;

    println!();

    let v: Value = serde_json::from_str(s).unwrap();

    if let Value::Object(map) = v {
        let _: Vec<_> = map.iter().map(|x| {
            let mut v_type = "obj";
            // println!("pub {}: {},", x.0, v_type);
            if x.1.is_i64() {
                v_type = "i64";
            } else if x.1.is_f64() {
                v_type = "f64";
            } else if x.1.is_boolean() {
                v_type = "bool";
            } else if x.1.is_string() {
                v_type = "String";
            } else if x.1.is_array() {
                v_type = "array";
            } else {
                v_type = "obj";
            }

            println!("pub {}: {},", x.0, v_type);
            x
        }).collect();
    }
    println!();
}