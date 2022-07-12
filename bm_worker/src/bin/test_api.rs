#![allow(dead_code)]

use actix_web::{rt};
use cffc_base::api::bm_api::{*};
use serde_json::{Result};

use serde::{Serialize, Deserialize};

use tokio::fs;


#[derive(Serialize, Deserialize, Debug)]
struct TestNoneStruct {
    url: String,
    id: Option<String>,
    ts: i64,
}

fn test_api_getsource() {
    let mut rt = rt::System::new("test");

    let api = AnalysisApi::new("http://192.168.1.220:7001");
    rt.block_on(async move {
        println!("enter block_on");
        let rst = api.get_sources().await;
        println!("{:?}", rst);

        let rst = api.get_source_info("ecb8bb2a-4abc-49b4-822f-14f2f3eb3f4f".to_string()).await;
        println!("{:?}", rst);
    });
}

fn test_api_serialize() {
    let req = TestNoneStruct {
        url: "abc".to_string(),
        id: Some("".to_string()),
        ts: 123,
    };

    let rst = serde_json::to_string(&req);
    println!("{:?}", rst);
}

fn test_api_deserialize() {
    let s = r#"
{
  "background": {
    "frame_num": 7892,
    "image": "",
    "image_file": "bg.jpg",
    "rect": {
      "h": 663,
      "w": 507,
      "x": 134,
      "y": 338
    },
    "width": 4096,
"height": 2160
  },
  "id": "4f354377-2e7e-4337-b475-d218209e9728",
  "index": 0,
  "plate_info": {
    "bits": [
      [
        {
          "conf": 0.5891571044921875,
          "value": "粤"
        },
        {
          "conf": 0.19207930564880371,
          "value": "辽"
        }
      ],
      [
        {
          "conf": 0.98767322301864624,
          "value": "B"
        },
        {
          "conf": 0.0020951344631612301,
          "value": "A"
        }
      ],
      [
        {
          "conf": 0.98114532232284546,
          "value": "9"
        },
        {
          "conf": 0.0020844351965934038,
          "value": "8"
        }
      ],
      [
        {
          "conf": 0.98693853616714478,
          "value": "B"
        },
        {
          "conf": 0.0049859574064612389,
          "value": "8"
        }
      ],
      [
        {
          "conf": 0.9272615909576416,
          "value": "R"
        },
        {
          "conf": 0.014163904823362827,
          "value": "8"
        }
      ],
      [
        {
          "conf": 0.99115324020385742,
          "value": "0"
        },
        {
          "conf": 0.0015164846554398537,
          "value": "D"
        }
      ],
      [
        {
          "conf": 0.98166090250015259,
          "value": "3"
        },
        {
          "conf": 0.0069634984247386456,
          "value": "7"
        }
      ]
    ],
    "image": "",
    "image_file": "plate.bmp",
    "text": "粤B9BR03",
    "type": {
      "conf": 0.99961990118026733,
      "value": "蓝牌单行"
    }
  },
  "position": {
    "end": 1598624224937,
    "end_frame": 7898,
    "start": 1598624222937,
    "start_frame": 7840,
    "start_real_time": 0,
"end_real_time": 0
  },
  "props": {
    "brand": [
      {
        "score": 0.27008199691772461,
        "value": "福特"
      }
    ],
    "color": [
      {
        "score": 0.99999988079071045,
        "value": "蓝色"
      }
    ],
    "direction": [
      {
        "score": 0.56151777505874634,
        "value": "前面"
      },
      {
        "score": 0.43553432822227478,
        "value": "侧前面"
      }
    ],
    "mid_type": [
      {
        "score": 0.99999988079071045,
        "value": "卡车"
      }
    ],
    "move_direction": 2,
    "series": [
      {
        "score": 0.30757012963294983,
        "value": "2015款 1.0L标准型厢式车单排XC4F18-T"
      }
    ],
    "top_series": [
      {
        "score": 0.32984122633934021,
        "value": "小金牛"
      },
      {
        "score": 0.23615778982639313,
        "value": "五菱宏光"
      }
    ],
    "top_type": [
      {
        "score": 0.99999988079071045,
        "value": "卡车"
      }
    ]
  },
  "source": "bb59da2d-dc08-4aed-a04f-765aecdc11bb",
  "vehicles": [
    {
      "frame_num": 7890,
      "image": "",
      "image_file": "768x766-0.jpg",
      "rect": {
        "h": 684,
        "w": 462,
        "x": 198,
        "y": 304
      }
    },
    {
      "frame_num": 7882,
      "image": "",
      "image_file": "564x562-1.jpg",
      "rect": {
        "h": 502,
        "w": 347,
        "x": 358,
        "y": 128
      }
    },
    {
      "frame_num": 7874,
      "image": "",
      "image_file": "460x460-2.jpg",
      "rect": {
        "h": 410,
        "w": 289,
        "x": 442,
        "y": 20
      }
    }
  ],
  "version": "1.1.7"
}

    "#;
    let req: Result<CarNotifyParams> = serde_json::from_str(s);

    println!("{:?}", req);
}

fn test_api_default_sourceconfig() {
    let mut guard = DEFAULT_CREATE_SOURCE_REQ_CONFIG.write().unwrap();
    guard.cfg.jpg_encode_threshold = 666;

    let mut config = guard.cfg.clone();
    config.jpg_encode_threshold = 888;
    println!("{:?}", config);


    println!("{:?}", guard.cfg);
}

fn test_api_detect() {
    let mut rt = rt::System::new("test");

    let api = RecognitionApi::new("http://192.168.1.220:7002");
    rt.block_on(async move {
        println!("enter block_on");

        let file_name = "/Users/tom/Desktop/tom.jpeg";
        // let file_name = "/Users/tom/Desktop/a.json";
        let data = fs::read(file_name).await.unwrap();
        let buf = base64::encode(data);

        let rst = api.detect(buf, true, true).await;
        println!("{:?}", rst);

        if let Ok(DetectRes { code, faces, .. }) = rst {
            if code == 0 {
                if let Some(faces) = faces {
                    let images: Vec<String> = faces.iter().map(|x| { x.aligned.clone() }).collect();

                    let rst = api.get_features(images, true).await;
                    println!("{:?}", rst);
                }
            }
        }
    });
}


fn test_api_createdb() {
    let mut rt = rt::System::new("test");

    let api = RecognitionApi::new("http://192.168.1.220:7002");
    rt.block_on(async move {
        println!("enter block_on");

        let rst = api.create_db(None, 50).await;
        println!("{:?}", rst);

        if let Ok(CreateDbRes { code, msg, id: Some(id) }) = rst {
            println!("code:{}, msg:{}, id: {}", code, msg, id);

            let rst = api.get_db_info(id).await;
            println!("{:?}", rst);
        }
    });
}

fn test_api_delete_db() {
    let mut rt = rt::System::new("test");

    let api = RecognitionApi::new("http://192.168.1.220:7002");
    rt.block_on(async move {
        println!("enter block_on");

        let rst = api.get_dbs().await;
        println!("{:?}", rst);

        let db_id = "00d2d403-2e81-4e9f-ba43-7facc3c619c9";

        let rst = api.flush_db(db_id.to_string()).await;
        println!("{:?}", rst);

        let rst = api.delete_db(db_id.to_string()).await;
        println!("{:?}", rst);
    });
}

fn test_api_create_one_person() {
    let mut rt = rt::System::new("test");

    let api = RecognitionApi::new("http://192.168.1.220:7002");
    rt.block_on(async move {
        println!("enter block_on");

        // let db = "c762bbe7-05aa-4f6e-b129-c2aaf6523229".to_string();
        let db = "bdcf3391-c4d1-4a89-89a8-ab1a9a7018f5".to_string();


        let file_name = "/Users/tom/Desktop/tom.jpeg";
        let data = fs::read(file_name).await.unwrap();
        let buf = base64::encode(data);

        let rst = api.detect(buf, true, true).await;
        println!("{:?}", rst);

        if let Ok(DetectRes { code, faces: Some(faces), .. }) = rst {
            if code != 0 {
                return;
            }
            let feas: Vec<ApiFeatureQuality> = faces.iter()
                .filter(|x| x.feature.is_some())
                .map(|x| ApiFeatureQuality {
                    feature: x.feature.as_ref().unwrap().clone(),
                    quality: x.score,
                }).collect();
            let ids = vec![];

            let mut features = Vec::new();
            features.push(feas);
            let rst = api.create_persons(db, ids, features).await;
            println!("{:?}", rst);
        }
    });
}


fn test_api_get_db_person() {
    let mut rt = rt::System::new("test");

    let api = RecognitionApi::new("http://192.168.1.220:7002");
    rt.block_on(async move {
        println!("enter block_on");

        let rst = api.get_dbs().await;
        println!("{:?}", rst);


        let db = "c762bbe7-05aa-4f6e-b129-c2aaf6523229".to_string();
        // let db = "bdcf3391-c4d1-4a89-89a8-ab1a9a7018f5".to_string();

        let rst = api.get_db_persons(db.clone(), 0, 100).await;
        println!("{:?}", rst);

        let id = "e2befe5e-9a8e-4b37-a09d-5aeb5eb01ff5".to_string();
        let rst = api.get_person_info(db.clone(), id).await;
        println!("{:?}", rst);
    });
}

fn test_api_delete_person() {
    let mut rt = rt::System::new("test");

    let api = RecognitionApi::new("http://192.168.1.220:7002");
    rt.block_on(async move {
        println!("enter block_on");

        let db = "c762bbe7-05aa-4f6e-b129-c2aaf6523229".to_string();
        let id = "e2befe5e-9a8e-4b37-a09d-5aeb5eb01ff5".to_string();

        let rst = api.delete_person(db, id).await;
        println!("{:?}", rst);
    });
}


fn test_api_add_features() {
    let mut rt = rt::System::new("test");

    let api = RecognitionApi::new("http://192.168.1.220:7002");
    rt.block_on(async move {
        println!("enter block_on");

        let db = "c762bbe7-05aa-4f6e-b129-c2aaf6523229".to_string();
        let id = "e2befe5e-9a8e-4b37-a09d-5aeb5eb01ff5".to_string();

        let file_name = "/Users/tom/Desktop/tom.jpeg";
        let data = fs::read(file_name).await.unwrap();
        let buf = base64::encode(data);

        let rst = api.detect(buf, true, true).await;
        println!("{:?}", rst);

        if let Ok(DetectRes { code, faces: Some(faces), .. }) = rst {
            if code != 0 {
                return;
            }
            let feas: Vec<ApiFeatureQuality> = faces.iter()
                .filter(|x| x.feature.is_some())
                .map(|x| ApiFeatureQuality {
                    feature: x.feature.as_ref().unwrap().clone(),
                    quality: x.score,
                }).collect();


            let rst = api.add_features_to_person(db, id, feas).await;
            println!("{:?}", rst);
        }
    });
}

fn test_api_add_aggregatefeatures() {
    let mut rt = rt::System::new("test");

    let api = RecognitionApi::new("http://192.168.1.220:7002");
    rt.block_on(async move {
        println!("enter block_on");

        let db = "c762bbe7-05aa-4f6e-b129-c2aaf6523229".to_string();
        let id = "e2befe5e-9a8e-4b37-a09d-5aeb5eb01ff5".to_string();

        let file_name = "/Users/tom/Desktop/tom.jpeg";
        let data = fs::read(file_name).await.unwrap();
        let buf = base64::encode(data);

        let rst = api.detect(buf, true, true).await;
        println!("{:?}", rst);

        if let Ok(DetectRes { code, faces: Some(faces), .. }) = rst {
            if code != 0 {
                return;
            }
            if let Some(ApiDetectFace { feature: Some(feature), .. }) = faces.get(0) {
                let rst = api.add_aggregate_feature_to_person(db, id, feature.clone()).await;
                println!("{:?}", rst);
            }
        }
    });
}


fn test_api_delete_person_feature() {
    let mut rt = rt::System::new("test");

    let api = RecognitionApi::new("http://192.168.1.220:7002");
    rt.block_on(async move {
        println!("enter block_on");

        let db = "c762bbe7-05aa-4f6e-b129-c2aaf6523229".to_string();
        let id = "e2befe5e-9a8e-4b37-a09d-5aeb5eb01ff5".to_string();
        let face_id = 2_i64;
        let rst = api.delete_person_feature(db, id, face_id).await;
        println!("{:?}", rst);
    });
}

#[allow(dead_code)]
fn test_api_move_persons() {
    let mut rt = rt::System::new("test");

    let api = RecognitionApi::new("http://192.168.1.220:7002");
    rt.block_on(async move {
        println!("enter block_on");

        let db_dst = "bdcf3391-c4d1-4a89-89a8-ab1a9a7018f5".to_string();
        let db_src = "c762bbe7-05aa-4f6e-b129-c2aaf6523229".to_string();
        let id = "e2befe5e-9a8e-4b37-a09d-5aeb5eb01ff5".to_string();

        let ids = vec![id];

        let rst = api.move_persons(db_src, db_dst, ids).await;

        println!("{:?}", rst);
    });
}

#[allow(dead_code)]
fn test_api_search() {
    let mut rt = rt::System::new("test");

    let api = RecognitionApi::new("http://192.168.1.220:7002");
    rt.block_on(async move {
        println!("enter block_on");

        let _db = "c762bbe7-05aa-4f6e-b129-c2aaf6523229".to_string();

        let file_name = "/Users/tom/Desktop/tom.jpeg";
        let data = fs::read(file_name).await.unwrap();
        let buf = base64::encode(data);

        let rst = api.detect(buf, true, true).await;
        println!("{:?}", rst);

        if let Ok(DetectRes { code, faces: Some(faces), .. }) = rst {
            if code != 0 {
                return;
            }
            let feas: Vec<ApiFeatureQuality> = faces.iter()
                .filter(|x| x.feature.is_some())
                .map(|x| ApiFeatureQuality {
                    feature: x.feature.as_ref().unwrap().clone(),
                    quality: x.score,
                }).collect();

            let mut features = Vec::new();
            features.push(feas);

            let db = vec!["926d00b1-ce50-41d6-9c66-4df096fec013", "56e6a47c-3d4d-4f99-b6a3-ca24028358df", "aee4a866-bda7-4019-9c37-8b98a37e4ad5", "c762bbe7-05aa-4f6e-b129-c2aaf6523229", "bdcf3391-c4d1-4a89-89a8-ab1a9a7018f5"];
            let db = db.iter().map(|x| x.to_string()).collect();
            let top = vec![10];
            let threshold = vec![98];


            let rst = api.search(db, top, threshold, features).await;
            println!("{:?}", rst);
        }
    });
}

#[allow(dead_code)]
fn test_api_compare() {
    let mut rt = rt::System::new("test");

    let api = RecognitionApi::new("http://192.168.1.220:7002");
    rt.block_on(async move {
        println!("enter block_on");

        let _db = "c762bbe7-05aa-4f6e-b129-c2aaf6523229".to_string();

        let file_name = "/Users/tom/Desktop/tom.jpeg";
        let data = fs::read(file_name).await.unwrap();
        let buf = base64::encode(data);

        let rst = api.detect(buf, true, true).await;
        println!("{:?}", rst);

        if let Ok(DetectRes { code, faces: Some(faces), .. }) = rst {
            if code != 0 {
                return;
            }

            let aligned_img: Vec<String> = faces.iter()
                .map(|x| x.aligned.clone()
                ).collect();

            if let Some(image) = aligned_img.get(0) {
                let rst = api.compare(image.clone(), image.clone()).await;
                println!("{:?}", rst);
            }
        }
    });
}

#[allow(dead_code)]
fn test_api_compare_n() {
    let mut rt = rt::System::new("test");

    let api = RecognitionApi::new("http://192.168.1.220:7002");
    rt.block_on(async move {
        println!("enter block_on");

        // let db = "c762bbe7-05aa-4f6e-b129-c2aaf6523229".to_string();
        let _db = "bdcf3391-c4d1-4a89-89a8-ab1a9a7018f5".to_string();


        let file_name = "/Users/tom/Desktop/tom.jpeg";
        let data = fs::read(file_name).await.unwrap();
        let buf = base64::encode(data);

        let rst = api.detect(buf, true, true).await;
        println!("{:?}", rst);

        if let Ok(DetectRes { code, faces: Some(faces), .. }) = rst {
            if code != 0 {
                return;
            }
            let feas: Vec<ApiFeatureQuality> = faces.iter()
                .filter(|x| x.feature.is_some())
                .map(|x| ApiFeatureQuality {
                    feature: x.feature.as_ref().unwrap().clone(),
                    quality: x.score,
                }).collect();

            let a: Vec<ApiFeatureQuality> = feas.iter().map(|x| ApiFeatureQuality {
                feature: x.feature.clone(),
                quality: x.quality,
            }).collect();

            let mut features = Vec::new();
            features.push(feas);
            let rst = api.compare_n(a, features).await;
            println!("{:?}", rst);
        }
    });
}


fn main() {
    println!("test_api");
    // test_api_serialize();
    // test_api_deserialize();
    // test_api_default_sourceconfig();
    test_api_detect();
    // test_api_createdb();
    // test_api_delete_db();

    // test_api_create_one_person();

    // test_api_delete_person();

    // test_api_add_features();

    // test_api_delete_person_feature();

    // test_api_get_db_person();

    // test_api_search();

    // test_api_move_persons();

    // test_api_add_aggregatefeatures();
    // test_api_compare();
    // test_api_compare_n();


    // test_api_getsource();
}