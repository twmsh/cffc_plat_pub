use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LanePoint {
    pub x: i64,
    pub y: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LaneLine {
    /// 靠上的点
    pub top: LanePoint,

    /// 靠下的点
    pub btm: LanePoint,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Lanes {
    pub lines: Vec<LaneLine>,
}

impl Lanes {
    /// 返回所在的车道号
    /// 从左往右数，从1开始, 返回0表示无效，表示在车道之外
    pub fn get_lane_num(&self, point: &LanePoint) -> usize {
        let mut index = -1;
        for (i, v) in self.lines.iter().enumerate() {
            // Tmp = (y1 – y2) * x + (x2 – x1) * y + x1 * y2 – x2 * y1
            // Tmp > 0 在左侧, Tmp = 0 在线上,  Tmp < 0 在右侧
            let tmp = (v.top.y - v.btm.y) * point.x + (v.btm.x - v.top.x) * point.y
                + v.top.x * v.btm.y - v.btm.x * v.top.y;

            if tmp < 0 {
                index = i as i64;
            } else {
                break;
            }
        }

        index += 1;

        // 如果index = 0 （车道左边界的左边） 或者 index = lines.len （车道的右边界的右边）
        // 表示在车道的有效范围之外, 返回 0
        if index == 0 || index >= self.lines.len() as i64 {
            return 0;
        }

        index as usize
    }

    /// 返回车辆的车道号
    /// 车道号从中间到路边计算（快车道向应急车道），编号从1开始
    /// vehicle_direct 表示摄像头与车辆行进方向的关系，false：摄像头对着车头（相对），true：摄像头对着车尾（同向）
    pub fn get_vehicle_lane_num(&self, point: &LanePoint, same_direct: bool) -> usize {
        let lane = self.get_lane_num(point);
        if lane == 0 || same_direct {
            return lane;
        }
        self.lines.len() - lane
    }

    /// 从json字符串中反序列化
    pub fn fromstr(s: &str) -> std::result::Result<Self, String> {
        serde_json::from_reader(s.as_bytes()).map_err(|x| format!("{}", x))
    }
}


pub fn get_vehicle_lane(x: i64, y: i64, lanes: &Lanes, same_direct: bool) -> usize {
    let car_center = LanePoint {
        x,
        y,
    };

    lanes.get_vehicle_lane_num(&car_center, same_direct)
}

pub fn get_vehicle_lane_fromstr(x: i64, y: i64, lanes_str: &str, same_direct: bool) -> std::result::Result<usize, String> {
    let lanes = Lanes::fromstr(lanes_str)?;
    Ok(get_vehicle_lane(x, y, &lanes, same_direct))
}
