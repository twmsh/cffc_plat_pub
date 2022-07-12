use cffc_base::api::bm_api;

use crate::dao::model::{CfDfnode, CfDfsource};
use crate::error::{AppError, AppResult};
use crate::web::proto::camera::{CameraItem, DfSrcParam};
use cffc_base::model::img_file;

/// 从 摄像头配置json中，提取srcparm相关字段
pub fn get_srcparam_from_json(src_url: &str, config_json: &str) -> AppResult<DfSrcParam> {
    let src_cfg: bm_api::CreateSourceReqConfig = serde_json::from_reader(config_json.as_bytes())?;

    Ok(DfSrcParam {
        url: src_url.to_string(),
        min_face: src_cfg.face.min_width,
        bg_width: src_cfg.produce.bg_width,
        area_top: src_cfg.face.top,
        area_left: src_cfg.face.left,
        area_width: src_cfg.face.width,
        area_height: src_cfg.face.height,
    })
}

/// 从node中获取节点，如果没有找到,err
/// 取出节点ip，如果是 localhost之类，用 defalut代替
fn find_node_ip(sid: &str, node_list: &[CfDfnode], default_value: &str) -> AppResult<String> {
    let node = node_list.iter().find(|x| x.node_sid.eq_ignore_ascii_case(sid));
    if node.is_none() {
        return Err(AppError::new(format!("can't find {}", sid).as_str()));
    }

    let node = node.unwrap();
    if node.ip.eq_ignore_ascii_case("localhost") || node.ip.eq_ignore_ascii_case("127.0.0.1")
        || node.ip.eq_ignore_ascii_case("0.0.0.0") {
        Ok(default_value.to_string())
    } else {
        Ok(node.ip.clone())
    }
}


/// 对 DfSrcParam  进行处理
/// 对 debug_url 进行处理，涉及到node ip
pub fn to_cameraitem_list(src_list: &[CfDfsource], node_list: &[CfDfnode],
                          app_ip: &str, live_port: i64) -> AppResult<Vec<CameraItem>> {
    let mut list = Vec::new();

    for v in src_list {
        let params = get_srcparam_from_json(v.src_url.as_str(), v.src_config.as_str())?;
        let rtsp_ip = find_node_ip(v.node_sid.as_str(), node_list, app_ip)?;
        let debug_url = img_file::get_debug_rtsp(rtsp_ip.as_str(), live_port, v.src_sid.as_str());

        list.push(CameraItem {
            id: v.id,
            src_sid: v.src_sid.clone(),
            name: v.name.clone(),
            node_sid: v.node_sid.clone(),
            src_url: v.src_url.clone(),
            push_url: v.push_url.clone(),
            ip: v.ip.clone(),
            src_state: v.src_state,
            src_config: v.src_config.clone(),
            grab_type: v.grab_type,
            io_flag: v.io_flag,
            direction: v.direction,
            tp_id: v.tp_id.clone(),
            upload_flag: v.upload_flag,
            location_name: v.location_name.clone(),
            resolution_ratio: v.resolution_ratio.clone(),
            coordinate: v.coordinate.clone(),
            sort_num: v.sort_num,
            trip_line: v.trip_line,
            rtcp_utc: v.rtcp_utc,
            lane_desc: v.lane_desc.clone(),
            lane_count: v.lane_count,
            memo: v.memo.clone(),
            gmt_create: v.gmt_create,
            gmt_modified: v.gmt_modified,
            params,
            debug_url,
        });
    }


    Ok(list)
}