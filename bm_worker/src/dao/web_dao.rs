use std::sync::Arc;

use log::debug;
use rusqlite::{NO_PARAMS, OptionalExtension, params};

use cffc_base::db::dbop::{DbOp, Result};
use cffc_base::db::SqliteClient;
use cffc_base::util::utils;
use cffc_base::util::utils::DateRange;

use crate::dao::model::*;

pub struct WebDao {
    pub client: Arc<SqliteClient>,
}

impl WebDao {
    pub fn new(client: Arc<SqliteClient>) -> Self {
        WebDao {
            client,
        }
    }

    pub fn load_dfnode_by_sid(&self, sid: &str) -> Result<Option<CfDfnode>> {
        let con = self.client.lock().unwrap();

        let sql = "select * from cf_dfnode where node_sid = ?";
        let v = con.query_row(sql, params![sid], |row| CfDfnode::scan(row)).optional()?;
        Ok(v)
    }

    pub fn load_beuser_by_loginname(&self, login_name: &str) -> Result<Option<BeUser>> {
        let con = self.client.lock().unwrap();

        let sql = "select * from be_user where login_name = ?";
        let v = con.query_row(sql, params![login_name], |row| BeUser::scan(row)).optional()?;
        Ok(v)
    }

    pub fn update_beuser_for_logon(&self, po: &BeUser) -> Result<usize> {
        let con = self.client.lock().unwrap();

        let sql = "update be_user set token = ?, ref_count = ?, last_login = ?, token_expire = ?, gmt_modified =? where login_name = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![po.token,po.ref_count,po.last_login,po.token_expire,po.gmt_modified,po.login_name])?;
        Ok(affect)
    }

    pub fn update_beuser_for_modify(&self, po: &BeUser) -> Result<usize> {
        let con = self.client.lock().unwrap();

        let sql = "update be_user set name = ?, password = ?, token = ?, phone = ?, email = ?, token_expire = ?, gmt_modified =? where login_name = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![po.name,po.password,po.token,po.phone,po.email, po.token_expire, po.gmt_modified,po.login_name])?;
        Ok(affect)
    }

    pub fn get_sourcelist_for_display(&self, limit: i64) -> Result<Vec<CfDfsource>> {
        let con = self.client.lock().unwrap();

        let sql = "select * from cf_dfsource where sort_num > 0 order by sort_num desc, gmt_modified desc limit ?";
        let mut stmt = con.prepare(sql)?;
        let mut rows = stmt.query(params![limit])?;

        let mut list = Vec::new();
        while let Some(row) = rows.next()? {
            let po = CfDfsource::scan(row)?;
            list.push(po);
        }
        Ok(list)
    }

    pub fn get_all_sourcelist(&self) -> Result<Vec<CfDfsource>> {
        let con = self.client.lock().unwrap();

        let sql = "select * from cf_dfsource order by id desc";
        let mut stmt = con.prepare(sql)?;
        let mut rows = stmt.query(NO_PARAMS)?;

        let mut list = Vec::new();
        while let Some(row) = rows.next()? {
            let po = CfDfsource::scan(row)?;
            list.push(po);
        }
        Ok(list)
    }

    pub fn get_dfnode_list(&self) -> Result<Vec<CfDfnode>> {
        let con = self.client.lock().unwrap();

        let sql = "select * from cf_dfnode";
        let mut stmt = con.prepare(sql)?;
        let mut rows = stmt.query(NO_PARAMS)?;

        let mut list = Vec::new();
        while let Some(row) = rows.next()? {
            let po = CfDfnode::scan(row)?;
            list.push(po);
        }
        Ok(list)
    }

    pub fn load_dfsource_by_sid(&self, sid: &str) -> Result<Option<CfDfsource>> {
        let con = self.client.lock().unwrap();

        let sql = "select * from cf_dfsource where src_sid = ?";
        let v = con.query_row(sql, params![sid], |row| CfDfsource::scan(row)).optional()?;
        Ok(v)
    }

    pub fn update_dfsource_for_onscreen(&self, po: &CfDfsource) -> Result<usize> {
        let con = self.client.lock().unwrap();

        let sql = "update cf_dfsource set sort_num = ?, gmt_modified = ? where src_sid = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![po.sort_num, po.gmt_modified,po.src_sid])?;
        Ok(affect)
    }

    pub fn update_dfsource_for_setstate(&self, po: &CfDfsource) -> Result<usize> {
        let con = self.client.lock().unwrap();

        let sql = "update cf_dfsource set src_state = ?, gmt_modified = ? where src_sid = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![po.src_state, po.gmt_modified,po.src_sid])?;
        Ok(affect)
    }

    pub fn load_dfsource_by_name(&self, name: &str) -> Result<Option<CfDfsource>> {
        let con = self.client.lock().unwrap();

        let sql = "select * from cf_dfsource where name = ?";
        let v = con.query_row(sql, params![name], |row| CfDfsource::scan(row)).optional()?;
        Ok(v)
    }

    pub fn save_dfsource_for_add(&self, po: &CfDfsource) -> Result<i64> {
        let mut con = self.client.lock().unwrap();
        po.insert(&mut con)
    }

    pub fn delete_dfsource_by_sid(&self, sid: &str) -> Result<usize> {
        let con = self.client.lock().unwrap();

        let sql = "delete from cf_dfsource where src_sid = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![sid])?;
        Ok(affect)
    }

    pub fn update_dfsource_for_modify(&self, po: &CfDfsource) -> Result<usize> {
        let con = self.client.lock().unwrap();

        let sql = "update cf_dfsource set name = ?, src_url = ?, ip = ?, src_state = ?, src_config = ?, grab_type = ?, gmt_modified = ? where src_sid = ? ";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![po.name,po.src_url,po.ip,po.src_state,po.src_config, po.grab_type, po.gmt_modified,po.src_sid])?;
        Ok(affect)
    }

    pub fn get_dfdb_list_for_poi(&self) -> Result<Vec<CfDfdb>> {
        let con = self.client.lock().unwrap();

        let sql = "select * from cf_dfdb where fp_flag = 1";
        let mut stmt = con.prepare(sql)?;
        let mut rows = stmt.query(NO_PARAMS)?;

        let mut list = Vec::new();
        while let Some(row) = rows.next()? {
            let po = CfDfdb::scan(row)?;
            list.push(po);
        }
        Ok(list)
    }

    pub fn load_cfpoi_by_sid(&self, sid: &str) -> Result<Option<CfPoi>> {
        let con = self.client.lock().unwrap();

        let sql = "select * from cf_poi where poi_sid = ?";
        let v = con.query_row(sql, params![sid], |row| CfPoi::scan(row)).optional()?;
        Ok(v)
    }

    pub fn save_poi(&self, po: &CfPoi) -> Result<i64> {
        let mut guard = self.client.lock().unwrap();
        po.insert(&mut guard)
    }


    pub fn get_dfdb_list(&self) -> Result<Vec<CfDfdb>> {
        let con = self.client.lock().unwrap();

        let sql = "select * from cf_dfdb";
        let mut stmt = con.prepare(sql)?;
        let mut rows = stmt.query(NO_PARAMS)?;

        let mut list = Vec::new();
        while let Some(row) = rows.next()? {
            let po = CfDfdb::scan(row)?;
            list.push(po);
        }
        Ok(list)
    }

    pub fn get_poi_total(&self, name: Option<String>, identity_card: Option<String>,
                         gender: Option<i64>,
                         group: Option<String>, threshold: Option<i64>) -> Result<Option<i64>> {
        let has_name = name.is_some();
        let has_identity = identity_card.is_some();
        let has_gender = gender.is_some();
        let has_group = group.is_some();
        let has_threshold = threshold.is_some();


        let mut vals: Vec<&dyn rusqlite::ToSql> = Vec::new();
        let mut sql = String::from("select count(*) from cf_poi t where 1=1 ");

        let name_like;
        let identity_like;
        if has_name {
            sql += " and t.name like ? ";
            name_like = format!("%{}%", name.unwrap());
            vals.push(&name_like);
        }
        if has_identity {
            sql += " and t.identity_card like ? ";
            identity_like = format!("%{}%", identity_card.unwrap());
            vals.push(&identity_like);
        }

        if has_gender {
            sql += " and t.gender = ? ";
            vals.push(&gender);
        }

        if has_group {
            sql += " and t.db_sid = ? ";
            vals.push(&group);
        }
        if has_threshold {
            sql += " and t.threshold >= ? ";
            vals.push(&threshold);
        }

        let con = self.client.lock().unwrap();
        let mut stmt = con.prepare(sql.as_str())?;
        let v = stmt.query_row(vals, |x| x.get(0)).optional()?;
        Ok(v)
    }

    pub fn get_poi_datapage(&self, name: Option<String>, identity_card: Option<String>,
                            gender: Option<i64>,
                            group: Option<String>, threshold: Option<i64>,
                            page_size: i64, start_index: i64) -> Result<Vec<CfPoi>> {
        let has_name = name.is_some();
        let has_identity = identity_card.is_some();
        let has_gender = gender.is_some();
        let has_group = group.is_some();
        let has_threshold = threshold.is_some();

        let mut vals: Vec<&dyn rusqlite::ToSql> = Vec::new();
        let mut sql = String::from("select id from cf_poi t where 1=1 ");

        let name_like;
        let identity_like;
        if has_name {
            sql += " and t.name like ? ";
            name_like = format!("%{}%", name.unwrap());
            vals.push(&name_like);
        }
        if has_identity {
            sql += " and t.identity_card like ? ";
            identity_like = format!("%{}%", identity_card.unwrap());
            vals.push(&identity_like);
        }
        if has_gender {
            sql += " and t.gender = ? ";
            vals.push(&gender);
        }
        if has_group {
            sql += " and t.db_sid = ? ";
            vals.push(&group);
        }
        if has_threshold {
            sql += " and t.threshold >= ? ";
            vals.push(&threshold);
        }

        sql += " order by t.id desc limit ?, ? ";
        vals.push(&start_index);
        vals.push(&page_size);

        let sql = format!("select a.* from cf_poi a join ( {} ) b on a.id = b.id order by a.id desc", sql);
        let con = self.client.lock().unwrap();
        let mut stmt = con.prepare(sql.as_str())?;
        let mut rows = stmt.query(vals)?;

        let mut list = Vec::new();
        while let Some(row) = rows.next()? {
            let po = CfPoi::scan(row)?;
            list.push(po);
        }
        Ok(list)
    }

    pub fn delete_cfpoi_by_sid(&self, sid: &str) -> Result<usize> {
        let con = self.client.lock().unwrap();

        let sql = "delete from cf_poi where poi_sid = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![sid])?;
        Ok(affect)
    }

    pub fn update_cfpoi_for_modify(&self, po: &CfPoi) -> Result<usize> {
        let con = self.client.lock().unwrap();

        let sql = "update cf_poi set name = ?, gender = ?, identity_card = ?, threshold = ?, feature_ids = ?, cover = ?, gmt_modified = ? where poi_sid = ? ";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![po.name,po.gender,po.identity_card,po.threshold,po.feature_ids, po.cover, po.gmt_modified,po.poi_sid])?;
        Ok(affect)
    }


    pub fn get_facetrack_total(&self, camera: Option<String>, date_range: Option<utils::DateRange>, alarm: Option<i64>,
                               name: Option<String>, identity_card: Option<String>,
                               gender: Option<i64>) -> Result<Option<i64>> {

        // select count(*) from cf_facetrack t where 1=1 and t.src_sid= ? and t.capture_time >= ? and t.capture_time < ?
        // and t.alarmed = ? and t.gender = ?

        // select count(*) from cf_facetrack t join (select poi_sid from cf_poi where name like ? and identity_card like ? ) a
        // on t.most_person =  a.poi_sid  where 1=1 and t.src_sid= ? and
        // t.capture_time >= ? and t.capture_time < ?
        // and t.alarmed = ? and t.gender = ?


        let has_camera = camera.is_some();
        let has_date = date_range.is_some();
        let has_alarm = alarm.is_some();
        let has_name = name.is_some();
        let has_identity = identity_card.is_some();
        let has_gender = gender.is_some();
        let has_join = has_name || has_identity;

        let mut vals: Vec<&dyn rusqlite::ToSql> = Vec::new();
        let mut sql = String::from("select count(*) from cf_facetrack t where 1=1 ");

        let name_like;
        let identity_like;
        let date_range_cl: DateRange;

        if has_join {
            sql = String::from("select count(*) from cf_facetrack t join (select poi_sid from cf_poi where 1=1 ");
            if has_name {
                sql += " and name like ? ";
                name_like = format!("%{}%", name.unwrap());
                vals.push(&name_like);
            }
            if has_identity {
                sql += " and identity_card like ? ";
                identity_like = format!("%{}%", identity_card.unwrap());
                vals.push(&identity_like);
            }

            sql += " ) a on t.most_person =  a.poi_sid where 1=1  "
        }

        if has_camera {
            sql += " and t.src_sid = ? ";
            vals.push(&camera);
        }

        if has_date {
            sql += " and t.capture_time >= ? and t.capture_time < ? ";
            date_range_cl = date_range.unwrap();
            vals.push(&date_range_cl.begin);
            vals.push(&date_range_cl.end);
        }

        if has_alarm {
            sql += " and t.alarmed = ? ";
            vals.push(&alarm);
        }

        if has_gender {
            sql += " and t.gender = ? ";
            vals.push(&gender);
        }
        debug!("sql: {}", sql);

        let con = self.client.lock().unwrap();
        let mut stmt = con.prepare(sql.as_str())?;
        let v = stmt.query_row(vals, |x| x.get(0)).optional()?;
        Ok(v)
    }

    pub fn get_facetrack_datapage(&self, camera: Option<String>, date_range: Option<utils::DateRange>,
                                  alarm: Option<i64>, name: Option<String>,
                                  identity_card: Option<String>, gender: Option<i64>,
                                  page_size: i64, start_index: i64) -> Result<Vec<CfFacetrack>> {

        //q := "select a.* from cf_poi a join () b on a.id = b.id order by a.id desc"

        let has_camera = camera.is_some();
        let has_date = date_range.is_some();
        let has_alarm = alarm.is_some();
        let has_name = name.is_some();
        let has_identity = identity_card.is_some();
        let has_gender = gender.is_some();
        let has_join = has_name || has_identity;

        let mut vals: Vec<&dyn rusqlite::ToSql> = Vec::new();
        let mut sql = String::from("select id from cf_facetrack t where 1=1 ");

        let name_like;
        let identity_like;
        let date_range_cl: DateRange;

        if has_join {
            sql = String::from("select id from cf_facetrack t join (select poi_sid from cf_poi where 1=1 ");
            if has_name {
                sql += " and name like ? ";
                name_like = format!("%{}%", name.unwrap());
                vals.push(&name_like);
            }
            if has_identity {
                sql += " and identity_card like ? ";
                identity_like = format!("%{}%", identity_card.unwrap());
                vals.push(&identity_like);
            }

            sql += " ) x on t.most_person =  x.poi_sid where 1=1  "
        }

        if has_camera {
            sql += " and t.src_sid = ? ";
            vals.push(&camera);
        }

        if has_date {
            sql += " and t.capture_time >= ? and t.capture_time < ? ";
            date_range_cl = date_range.unwrap();
            vals.push(&date_range_cl.begin);
            vals.push(&date_range_cl.end);
        }

        if has_alarm {
            sql += " and t.alarmed = ? ";
            vals.push(&alarm);
        }

        if has_gender {
            sql += " and t.gender = ? ";
            vals.push(&gender);
        }

        sql += " order by t.id desc limit ?, ? ";
        vals.push(&start_index);
        vals.push(&page_size);

        let sql = format!("select a.* from cf_facetrack a join ( {} ) b on a.id = b.id order by a.id desc", sql);
        debug!("sql: {}", sql);

        let con = self.client.lock().unwrap();
        let mut stmt = con.prepare(sql.as_str())?;
        let mut rows = stmt.query(vals)?;

        let mut list = Vec::new();
        while let Some(row) = rows.next()? {
            let po = CfFacetrack::scan(row)?;
            list.push(po);
        }
        Ok(list)
    }

    pub fn get_coigroup_list(&self) -> Result<Vec<CfCoiGroup>> {
        let con = self.client.lock().unwrap();

        let sql = "select * from cf_coi_group";
        let mut stmt = con.prepare(sql)?;
        let mut rows = stmt.query(NO_PARAMS)?;

        let mut list = Vec::new();
        while let Some(row) = rows.next()? {
            let po = CfCoiGroup::scan(row)?;
            list.push(po);
        }
        Ok(list)
    }


    pub fn get_cartrack_total(&self, camera: Option<String>, date_range: Option<utils::DateRange>, alarm: Option<i64>,
                              plate_content: Option<String>, plate_type: Option<String>,
                              car_type: Option<String>, car_color: Option<String>) -> Result<Option<i64>> {
        let has_camera = camera.is_some();
        let has_date = date_range.is_some();
        let has_alarm = alarm.is_some();
        let mut has_plate_content = plate_content.is_some();
        let mut has_plate_type = plate_type.is_some();
        let has_car_type = car_type.is_some();
        let has_car_color = car_color.is_some();

        let date_range_cl: DateRange;
        let plate_content_like;

        let mut has_miss_plate = false; // 无车牌 /没有识别出车牌
        let mut vals: Vec<&dyn rusqlite::ToSql> = Vec::new();
        let mut sql = String::from("select count(*) from cf_cartrack t where 1=1 ");

        if let Some(ref v) = plate_type {
            if v.eq("无车牌") {
                has_miss_plate = true;
                has_plate_content = false;
                has_plate_type = false;
            }
        }

        if has_camera {
            sql += " and t.src_sid = ? ";
            vals.push(&camera);
        }

        if has_date {
            sql += " and t.capture_time >= ? and t.capture_time < ? ";
            date_range_cl = date_range.unwrap();
            vals.push(&date_range_cl.begin);
            vals.push(&date_range_cl.end);
        }

        if has_alarm {
            sql += " and t.alarmed = ? ";
            vals.push(&alarm);
        }

        if has_plate_content {
            sql += " and t.plate_content like ? ";
            plate_content_like = format!("%{}%", plate_content.unwrap());
            vals.push(&plate_content_like);
        }

        if has_plate_type {
            sql += " and t.plate_type = ? ";
            vals.push(&plate_type);
        }

        if has_car_type {
            sql += " and t.car_top_type = ? ";
            vals.push(&car_type);
        }

        if has_car_color {
            sql += " and t.car_color = ? ";
            vals.push(&car_color);
        }

        if has_miss_plate {
            sql += " and t.plate_judged = 0 ";
        }
        debug!("sql: {}", sql);

        let con = self.client.lock().unwrap();
        let mut stmt = con.prepare(sql.as_str())?;
        let v = stmt.query_row(vals, |x| x.get(0)).optional()?;
        Ok(v)
    }


    pub fn get_cartrack_datapage(&self, camera: Option<String>, date_range: Option<utils::DateRange>, alarm: Option<i64>,
                                 plate_content: Option<String>, plate_type: Option<String>,
                                 car_type: Option<String>, car_color: Option<String>,
                                 page_size: i64, start_index: i64) -> Result<Vec<CfCartrack>> {
        let has_camera = camera.is_some();
        let has_date = date_range.is_some();
        let has_alarm = alarm.is_some();
        let mut has_plate_content = plate_content.is_some();
        let mut has_plate_type = plate_type.is_some();
        let has_car_type = car_type.is_some();
        let has_car_color = car_color.is_some();

        let date_range_cl: DateRange;
        let plate_content_like;

        let mut has_miss_plate = false; // 无车牌 /没有识别出车牌
        let mut vals: Vec<&dyn rusqlite::ToSql> = Vec::new();
        let mut sql = String::from("select id from cf_cartrack t where 1=1 ");

        if let Some(ref v) = plate_type {
            if v.eq("无车牌") {
                has_miss_plate = true;
                has_plate_content = false;
                has_plate_type = false;
            }
        }

        if has_camera {
            sql += " and t.src_sid = ? ";
            vals.push(&camera);
        }

        if has_date {
            sql += " and t.capture_time >= ? and t.capture_time < ? ";
            date_range_cl = date_range.unwrap();
            vals.push(&date_range_cl.begin);
            vals.push(&date_range_cl.end);
        }

        if has_alarm {
            sql += " and t.alarmed = ? ";
            vals.push(&alarm);
        }

        if has_plate_content {
            sql += " and t.plate_content like ? ";
            plate_content_like = format!("%{}%", plate_content.unwrap());
            vals.push(&plate_content_like);
        }

        if has_plate_type {
            sql += " and t.plate_type = ? ";
            vals.push(&plate_type);
        }

        if has_car_type {
            sql += " and t.car_top_type = ? ";
            vals.push(&car_type);
        }

        if has_car_color {
            sql += " and t.car_color = ? ";
            vals.push(&car_color);
        }

        if has_miss_plate {
            sql += " and t.plate_judged = 0 ";
        }
        sql += " order by t.id desc limit ?, ? ";
        vals.push(&start_index);
        vals.push(&page_size);

        let sql = format!("select a.* from cf_cartrack a join ( {} ) b on a.id = b.id order by a.id desc", sql);
        debug!("sql: {}", sql);

        let con = self.client.lock().unwrap();
        let mut stmt = con.prepare(sql.as_str())?;
        let mut rows = stmt.query(vals)?;

        let mut list = Vec::new();
        while let Some(row) = rows.next()? {
            let po = CfCartrack::scan(row)?;
            list.push(po);
        }
        Ok(list)
    }

    pub fn load_cfcoi_by_plate(&self, plate: &str) -> Result<Option<CfCoi>> {
        let con = self.client.lock().unwrap();

        let sql = "select * from cf_coi where plate_content = ?";
        let v = con.query_row(sql, params![plate], |row| CfCoi::scan(row)).optional()?;
        Ok(v)
    }


    pub fn get_coi_total(&self, name: Option<String>, plate: Option<String>,
                         phone: Option<String>,
                         group: Option<String>) -> Result<Option<i64>> {
        let has_name = name.is_some();
        let has_plate = plate.is_some();
        let has_phone = phone.is_some();
        let has_group = group.is_some();


        let mut vals: Vec<&dyn rusqlite::ToSql> = Vec::new();
        let mut sql = String::from("select count(*) from cf_coi t where 1=1 ");

        let name_like;
        let plate_like;
        if has_name {
            sql += " and t.owner_name like ? ";
            name_like = format!("%{}%", name.unwrap());
            vals.push(&name_like);
        }
        if has_plate {
            sql += " and t.plate_content like ? ";
            plate_like = format!("%{}%", plate.unwrap());
            vals.push(&plate_like);
        }

        if has_phone {
            sql += " and t.owner_phone = ? ";
            vals.push(&phone);
        }

        if has_group {
            sql += " and t.group_sid = ? ";
            vals.push(&group);
        }

        let con = self.client.lock().unwrap();
        let mut stmt = con.prepare(sql.as_str())?;
        let v = stmt.query_row(vals, |x| x.get(0)).optional()?;
        Ok(v)
    }


    pub fn get_coi_datapage(&self, name: Option<String>, plate: Option<String>,
                            phone: Option<String>, group: Option<String>,
                            page_size: i64, start_index: i64) -> Result<Vec<CfCoi>> {
        let has_name = name.is_some();
        let has_plate = plate.is_some();
        let has_phone = phone.is_some();
        let has_group = group.is_some();

        let name_like;
        let plate_like;

        let mut vals: Vec<&dyn rusqlite::ToSql> = Vec::new();
        let mut sql = String::from("select id from cf_coi t where 1=1 ");

        if has_name {
            sql += " and t.owner_name like ? ";
            name_like = format!("%{}%", name.unwrap());
            vals.push(&name_like);
        }
        if has_plate {
            sql += " and t.plate_content like ? ";
            plate_like = format!("%{}%", plate.unwrap());
            vals.push(&plate_like);
        }

        if has_phone {
            sql += " and t.owner_phone = ? ";
            vals.push(&phone);
        }

        if has_group {
            sql += " and t.group_sid = ? ";
            vals.push(&group);
        }

        sql += " order by t.id desc limit ?, ? ";
        vals.push(&start_index);
        vals.push(&page_size);

        let sql = format!("select a.* from cf_coi a join ( {} ) b on a.id = b.id order by a.id desc", sql);
        debug!("sql: {}", sql);

        let con = self.client.lock().unwrap();
        let mut stmt = con.prepare(sql.as_str())?;
        let mut rows = stmt.query(vals)?;

        let mut list = Vec::new();
        while let Some(row) = rows.next()? {
            let po = CfCoi::scan(row)?;
            list.push(po);
        }
        Ok(list)
    }

    pub fn load_cfcoi_by_sid(&self, sid: &str) -> Result<Option<CfCoi>> {
        let con = self.client.lock().unwrap();

        let sql = "select * from cf_coi where sid = ?";
        let v = con.query_row(sql, params![sid], |row| CfCoi::scan(row)).optional()?;
        Ok(v)
    }

    pub fn save_cfcoi_for_add(&self, po: &CfCoi) -> Result<i64> {
        let mut con = self.client.lock().unwrap();
        po.insert(&mut con)
    }

    pub fn delete_cfcoi_by_sid(&self, sid: &str) -> Result<usize> {
        let con = self.client.lock().unwrap();

        let sql = "delete from cf_coi where sid = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![sid])?;
        Ok(affect)
    }

    pub fn update_cfcoi_for_modify(&self, po: &CfCoi) -> Result<usize> {
        let con = self.client.lock().unwrap();

        let sql = "update cf_coi set group_sid = ?, plate_content = ?, plate_type = ?, owner_name = ?, owner_phone = ?, memo = ?, gmt_modified = ? where sid = ? ";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![po.group_sid,po.plate_content,po.plate_type,po.owner_name,po.owner_phone,po.memo, po.gmt_modified,po.sid])?;
        Ok(affect)
    }


    pub fn load_latest_facetrack_alarm_list(&self, limit: i64) -> Result<Vec<CfFacetrack>> {
        let con = self.client.lock().unwrap();

        let sql = "select * from cf_facetrack where alarmed = 1 order by id desc limit ?";
        let mut stmt = con.prepare(sql)?;
        let mut rows = stmt.query(params![limit])?;

        let mut list = Vec::new();
        while let Some(row) = rows.next()? {
            let po = CfFacetrack::scan(row)?;
            list.push(po);
        }
        Ok(list)
    }

    pub fn load_latest_cartrack_alarm_list(&self, limit: i64) -> Result<Vec<CfCartrack>> {
        let con = self.client.lock().unwrap();

        let sql = "select * from cf_cartrack where alarmed = 1 order by id desc limit ?";
        let mut stmt = con.prepare(sql)?;
        let mut rows = stmt.query(params![limit])?;

        let mut list = Vec::new();
        while let Some(row) = rows.next()? {
            let po = CfCartrack::scan(row)?;
            list.push(po);
        }
        Ok(list)
    }
}