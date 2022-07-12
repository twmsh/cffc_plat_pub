use bytes::Buf;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

use cffc_base::api::bm_api::{CarNotifyParams, FaceNotifyParams};
use cffc_base::model::img_file;

use crate::dao::model::{CfCartrack, CfCoi, CfCoiGroup, CfDfdb, CfDfsource, CfFacetrack, CfPoi};
use crate::error::{AppError, AppResult};

// ------------------- queue structs (face) -------------------

#[derive(Debug)]
pub struct NotifyFaceQueueItem {
    pub uuid: String,
    pub notify: FaceNotifyParams,
    pub ts: DateTime<Local>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CameraQI {
    pub id: i64,
    pub sid: String,
    pub name: String,
    pub url: String,
    pub state: i32,
    pub memo: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FtQIFace {
    pub index: i64,
    pub feature: Option<String>,
    pub quality: f64,
    pub s_img_url: String,
    pub l_img_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FtQIFaceProps {
    pub age: i64,
    pub gender: i64,
    pub glasses: i64,
    pub direction: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FtQIFaces {
    pub sid: String,
    pub source: String,
    pub faces: Vec<FtQIFace>,
    pub props: Option<FtQIFaceProps>,

    pub bg_url: String,
    pub ts: DateTime<Local>,
    pub matched: bool,
    pub judged: bool,
    pub alarmed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FtQIPerson {
    pub id: i64,
    pub sid: String,
    pub name: String,
    pub id_card: String,
    pub gender: i64,
    pub cover: i64,
    pub cover_url: String,
    pub imgs_url: Vec<String>,
    pub threshold: i64,
    pub score: i64,
    pub db_sid: String,
    pub db_name: String,
    pub bw_flag: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FtQI {
    pub sid: String,
    pub face: FtQIFaces,
    pub camera: Option<CameraQI>,
    pub match_poi: Option<FtQIPerson>,
}

impl FtQIFaces {
    fn from_po(url_prefix: &str, po: &CfFacetrack) -> AppResult<Self> {
        let mut faces = Vec::new();
        let ids = img_file::get_item_from_idscores(po.img_ids.as_str())?;

        ids.iter().for_each(|x| {
            faces.push(FtQIFace {
                index: x.0,
                feature: None,
                quality: x.1,
                s_img_url: img_file::get_facetrack_smallimg_url(url_prefix, &po.ft_sid, x.0),
                l_img_url: img_file::get_facetrack_largeimg_url(url_prefix, &po.ft_sid, x.0),
            });
        });


        let props = Some(FtQIFaceProps {
            age: po.age.map_or(0, |x| x as i64),
            gender: po.gender.map_or(0, |x| x as i64),
            glasses: po.glasses.map_or(0, |x| x as i64),
            direction: po.direction.map_or(0, |x| x as i64),
        });


        Ok(FtQIFaces {
            sid: po.ft_sid.clone(),
            source: po.src_sid.clone(),
            faces,
            props,
            bg_url: img_file::get_facetrack_bgimg_url(url_prefix, &po.ft_sid),
            ts: po.capture_time,
            matched: po.matched.map_or(false, |x| x == 1),
            judged: po.judged.map_or(false, |x| x == 1),
            alarmed: po.alarmed.map_or(false, |x| x == 1),
        })
    }
}

impl CameraQI {
    fn from_po(po: &CfDfsource) -> Self {
        CameraQI {
            id: po.id,
            sid: po.src_sid.clone(),
            name: po.name.clone(),
            url: po.src_url.clone(),
            state: po.src_state,
            memo: po.memo.clone(),
        }
    }
}

impl CtQIPerson {
    fn from_po(po: &CfCoi, group: &CfCoiGroup) -> Self {
        CtQIPerson {
            id: po.id,
            sid: po.sid.clone(),
            plate_content: po.plate_content.clone(),
            plate_type: po.plate_type.clone(),
            owner_name: po.owner_name.clone(),
            owner_phone: po.owner_phone.clone(),
            owner_address: po.owner_address.clone(),
            group_sid: group.sid.clone(),
            group_name: group.name.clone(),
            bw_flag: group.bw_flag as i64,
        }
    }
}


impl FtQIPerson {
    fn from_po(url_prefix: &str, po: &CfPoi, score: f64, db: &CfDfdb) -> AppResult<Self> {
        let mut imgs_url = Vec::new();
        let ids = img_file::get_item_from_idscores(&po.feature_ids)?;

        ids.iter().for_each(|x| {
            imgs_url.push(img_file::get_person_img_url(url_prefix, &po.poi_sid, x.0));
        });


        Ok(FtQIPerson {
            id: po.id,
            sid: po.poi_sid.clone(),
            name: po.name.clone(),
            id_card: po.identity_card.as_ref().map_or("".to_string(), |x| x.clone()),
            gender: po.gender.map_or(0, |x| x as i64),
            cover: po.cover.map_or(0, |x| x as i64),
            cover_url: po.cover.map_or("".to_string(), |_x| {
                img_file::get_person_cover_url(url_prefix, &po.poi_sid)
            }),
            imgs_url,
            threshold: po.threshold as i64,
            score: score as i64,
            db_sid: db.db_sid.clone(),
            db_name: db.name.clone(),
            bw_flag: db.bw_flag as i64,
        })
    }
}


impl FtQI {
    pub fn from_notify(url_prefix: &str, ts: DateTime<Local>, notify: &FaceNotifyParams, source_po: &Option<CfDfsource>) -> Self {
        let camera = source_po.as_ref().map(|x| CameraQI::from_po(x));

        let mut img_num = 0;
        let mut faces = Vec::new();
        for v in notify.faces.iter() {
            img_num += 1;

            let feature = match v.feature_buf {
                Some(ref v) => {
                    Some(base64::encode(v.bytes()))
                }
                None => None,
            };

            faces.push(FtQIFace {
                index: img_num,
                feature,
                quality: v.quality,
                s_img_url: img_file::get_facetrack_smallimg_url(url_prefix, &notify.id, img_num),
                l_img_url: img_file::get_facetrack_largeimg_url(url_prefix, &notify.id, img_num),
            });
        }

        let props = match notify.props {
            Some(ref v) => {
                Some(FtQIFaceProps {
                    age: v.age,
                    gender: v.gender,
                    glasses: v.glasses,
                    direction: v.move_direction,
                })
            }
            None => None,
        };

        FtQI {
            sid: notify.id.clone(),
            face: FtQIFaces {
                sid: notify.id.clone(),
                source: notify.source.clone(),
                faces,
                props,
                bg_url: img_file::get_facetrack_bgimg_url(url_prefix, &notify.id),
                ts,
                matched: false,
                judged: false,
                alarmed: false,
            },
            camera,
            match_poi: None,
        }
    }

    pub fn from_po(url_prefix: &str, po: &CfFacetrack, camera: Option<&CfDfsource>, db_list: &Vec<CfDfdb>, match_poi: Option<CfPoi>) -> AppResult<Self> {
        let qi_camera = match camera {
            Some(v) => {
                Some(CameraQI::from_po(v))
            }
            None => None,
        };

        let mut qi_match = None;
        if let Some(ref poi) = match_poi {
            let db = db_list.iter().find_map(|x| {
                if x.db_sid.eq(&poi.db_sid) {
                    Some(x)
                } else {
                    None
                }
            });
            if db.is_none() {
                return Err(AppError::new(&format!("can't find dfdb:{}", poi.db_sid)));
            }
            let db = db.unwrap();
            qi_match = Some(FtQIPerson::from_po(url_prefix, poi, po.most_score.map_or(0_f64, |x| x), db)?)
        }

        let qi_faces = FtQIFaces::from_po(url_prefix, po)?;

        Ok(FtQI {
            sid: po.ft_sid.clone(),
            face: qi_faces,
            camera: qi_camera,
            match_poi: qi_match,
        })
    }
}


// ------------------- queue structs (car) -------------------
#[derive(Debug)]
pub struct NotifyCarQueueItem {
    pub uuid: String,
    pub notify: CarNotifyParams,
    pub ts: DateTime<Local>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CtQICarPlate {
    pub content: String,
    pub plate_type: String,
    // pub type_score: f64,
    pub img_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CtQICarProps {
    /// 车身颜色
    pub color: String,

    /// 品牌
    pub brand: String,

    /// 车系
    pub top_series: String,

    /// 车款
    pub series: String,

    /// 车粗分类别
    pub top_type: String,

    /// 车类别
    pub mid_type: String,

    /// 车辆方向
    pub direct: String,

    /// 运动方向，0 未知；1 向上；2 向下
    pub move_direct: i64,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CtQICar {
    pub sid: String,
    pub source: String,
    pub img_urls: Vec<String>,

    pub plate: Option<CtQICarPlate>,
    pub props: Option<CtQICarProps>,

    pub bg_url: String,
    pub ts: DateTime<Local>,
    pub alarmed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CtQIPerson {
    pub id: i64,
    pub sid: String,
    pub plate_content: String,
    pub plate_type: Option<String>,
    pub owner_name: Option<String>,
    pub owner_phone: Option<String>,
    pub owner_address: Option<String>,
    pub group_sid: String,
    pub group_name: String,
    pub bw_flag: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CtQI {
    pub sid: String,
    pub car: CtQICar,
    pub camera: Option<CameraQI>,
    pub match_coi: Option<CtQIPerson>,
}

impl CtQICar {
    pub fn from_po(url_prefix: &str, po: &CfCartrack) -> AppResult<Self> {
        let mut img_urls = Vec::new();
        let ids = img_file::get_item_from_idscores(po.img_ids.as_str())?;

        ids.iter().for_each(|x| {
            let img_url = img_file::get_cartrack_img_url(url_prefix, &po.sid, x.0);
            img_urls.push(img_url);
        });

        let mut qi_plate = None;
        if po.plate_judged == 1 {
            qi_plate = Some(CtQICarPlate {
                content: po.plate_content.as_ref().map_or("".to_string(), |x| x.clone()),
                plate_type: po.plate_type.as_ref().map_or("".to_string(), |x| x.clone()),
                img_url: img_file::get_cartrack_plate_url(url_prefix, &po.sid),
            });
        }

        let mut qi_props = None;
        if po.vehicle_judged == 1 {
            qi_props = Some(CtQICarProps {
                color: po.car_color.as_ref().map_or("".to_string(), |x| x.clone()),
                brand: po.car_brand.as_ref().map_or("".to_string(), |x| x.clone()),
                top_series: po.car_top_series.as_ref().map_or("".to_string(), |x| x.clone()),
                series: po.car_series.as_ref().map_or("".to_string(), |x| x.clone()),
                top_type: po.car_top_type.as_ref().map_or("".to_string(), |x| x.clone()),
                mid_type: po.car_mid_type.as_ref().map_or("".to_string(), |x| x.clone()),
                direct: po.car_direct.as_ref().map_or("".to_string(), |x| x.clone()),
                move_direct: po.move_direct as i64,
            });
        }

        Ok(CtQICar {
            sid: po.sid.clone(),
            source: po.src_sid.clone(),
            img_urls,
            plate: qi_plate,
            props: qi_props,
            bg_url: img_file::get_cartrack_bgimg_url(url_prefix, &po.sid),
            ts: po.capture_time,
            alarmed: po.alarmed == 1,
        })
    }
}

impl CtQI {
    pub fn from_notify(url_prefix: &str, ts: DateTime<Local>, notify: &CarNotifyParams, source_po: &Option<CfDfsource>) -> Self {
        let camera = source_po.as_ref().map(|x| CameraQI::from_po(x));

        let mut img_num = 0;
        let mut img_urls = Vec::new();
        for _v in notify.vehicles.iter() {
            img_num += 1;
            img_urls.push(img_file::get_cartrack_img_url(url_prefix, &notify.id, img_num));
        }

        let mut props = None;
        if notify.has_props_info() {
            let (move_direct, direct, color, brand,
                top_series, series, top_type, mid_type) = notify.get_props_tuple();
            props = Some(CtQICarProps {
                color: color.unwrap_or_default(),
                brand: brand.unwrap_or_default(),
                top_series: top_series.unwrap_or_default(),
                series: series.unwrap_or_default(),
                top_type: top_type.unwrap_or_default(),
                mid_type: mid_type.unwrap_or_default(),
                direct: direct.unwrap_or_default(),
                move_direct: move_direct as i64,
            });
        }

        let mut plate = None;
        if notify.has_plate_info() {
            let (plate_content, plate_type) = notify.get_plate_tuple();
            plate = Some(CtQICarPlate {
                content: plate_content.unwrap_or_default(),
                plate_type: plate_type.unwrap_or_default(),
                img_url: img_file::get_cartrack_plate_url(url_prefix, &notify.id),
            });
        }

        CtQI {
            sid: notify.id.clone(),
            car: CtQICar {
                sid: notify.id.clone(),
                source: notify.source.clone(),
                img_urls,
                plate,
                props,
                bg_url: img_file::get_cartrack_bgimg_url(url_prefix, &notify.id),
                ts,
                alarmed: false,
            },
            camera,
            match_coi: None,
        }
    }

    pub fn from_po(url_prefix: &str, po: &CfCartrack, camera: Option<&CfDfsource>, group_list: &Vec<CfCoiGroup>, match_coi: Option<CfCoi>) -> AppResult<Self> {
        let qi_camera = match camera {
            Some(v) => {
                Some(CameraQI::from_po(v))
            }
            None => None,
        };

        let mut qi_match = None;
        if let Some(ref coi) = match_coi {
            let db = group_list.iter().find_map(|x| {
                if x.sid.eq(&coi.group_sid) {
                    Some(x)
                } else {
                    None
                }
            });
            if db.is_none() {
                return Err(AppError::new(&format!("can't find coi_group:{}", coi.group_sid)));
            }
            let db = db.unwrap();
            qi_match = Some(CtQIPerson::from_po(coi, db))
        }

        let qi_car = CtQICar::from_po(url_prefix, po)?;

        Ok(CtQI {
            sid: po.sid.clone(),
            car: qi_car,
            camera: qi_camera,
            match_coi: qi_match,
        })
    }
}

// ------------------- queue structs (general) -------------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum QI {
    #[serde(rename = "ft")]
    FT(FtQI),

    #[serde(rename = "ct")]
    CT(Box<CtQI>),
}

impl QI {
    pub fn get_sid(&self) -> String {
        match self {
            QI::FT(v) => v.sid.clone(),
            QI::CT(v) => v.sid.clone(),
        }
    }

    pub fn get_type(&self) -> i64 {
        match self {
            QI::FT(_) => 0,
            QI::CT(_) => 1,
        }
    }
}
