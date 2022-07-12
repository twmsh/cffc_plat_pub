use std::sync::{Arc};

use rusqlite::{OptionalExtension, params, NO_PARAMS};

use cffc_base::db::{
    SqliteClient,
    dbop::{DbOp, Result}};

use crate::dao::model::{CfCartrack, CfDfsource, CfFacetrack, CfPoi, CfCoi};

pub mod model;
pub mod web_dao;

pub struct AppDao {
    pub client: Arc<SqliteClient>,
    // pub conn: Mutex<rusqlite::Connection>,
}

impl AppDao {
    pub fn new(client: Arc<SqliteClient>) -> Self {
        AppDao {
            client,
        }
    }

    pub fn save_facetrack(&self, po: &CfFacetrack) -> Result<i64> {
        let mut guard = self.client.lock().unwrap();
        po.insert(&mut guard)
    }

    pub fn upate_facetrack_for_append(&self, po: &CfFacetrack) -> Result<usize> {
        let con = self.client.lock().unwrap();

        let sql = "update cf_facetrack set img_ids = ?, gender = ?, age = ?, glasses = ?, direction = ?, gmt_modified = ? where ft_sid = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![po.img_ids,po.gender,po.age,po.glasses,po.direction,po.gmt_modified,po.ft_sid])?;
        Ok(affect)
    }

    pub fn load_source_by_sid(&self, sid: &str) -> Result<Option<CfDfsource>> {
        let con = self.client.lock().unwrap();

        let sql = "select * from cf_dfsource where src_sid = ?";
        let v = con.query_row(sql, params![sid], |row| CfDfsource::scan(row)).optional()?;
        Ok(v)
    }

    /// 查找最旧的facetrack记录
    pub fn load_eldest_ft(&self, limit: i64) -> Result<Vec<(i64, String)>> {
        let con = self.client.lock().unwrap();
        let sql = "select id,ft_sid from cf_facetrack order by id asc limit ?";
        let mut stmt = con.prepare(sql)?;
        let mut rows = stmt.query(params![limit])?;

        let mut ids = Vec::new();
        while let Some(row) = rows.next()? {
            let id: i64 = row.get("id")?;
            let sid: String = row.get("ft_sid")?;
            ids.push((id, sid));
        }

        Ok(ids)
    }

    /// 删除 <= id 的facetrack记录
    pub fn delete_eldest_ft(&self, id: i64) -> Result<usize> {
        let con = self.client.lock().unwrap();

        let sql = "delete from cf_facetrack where id <= ?";
        let affect = con.execute(sql, params![id])?;
        Ok(affect)
    }


    /// 查找最旧的cartrack记录
    pub fn load_eldest_ct(&self, limit: i64) -> Result<Vec<(i64, String)>> {
        let con = self.client.lock().unwrap();
        let sql = "select id,sid from cf_cartrack order by id asc limit ?";
        let mut stmt = con.prepare(sql)?;
        let mut rows = stmt.query(params![limit])?;

        let mut ids = Vec::new();
        while let Some(row) = rows.next()? {
            let id: i64 = row.get("id")?;
            let sid: String = row.get("sid")?;
            ids.push((id, sid));
        }

        Ok(ids)
    }

    /// 删除 <= id 的cartrack记录
    pub fn delete_eldest_ct(&self, id: i64) -> Result<usize> {
        let con = self.client.lock().unwrap();

        let sql = "delete from cf_cartrack where id <= ?";
        let affect = con.execute(sql, params![id])?;
        Ok(affect)
    }


    pub fn load_automatch_dbs(&self) -> Result<Vec<(String, String, i32)>> {
        let con = self.client.lock().unwrap();

        let sql = "select db_sid,name,bw_flag from cf_dfdb where auto_match = ? and fp_flag = ?";
        let mut stmt = con.prepare(sql)?;
        let mut rows = stmt.query(params![1,1])?;

        let mut ids = Vec::new();
        while let Some(row) = rows.next()? {
            let db_sid: String = row.get("db_sid")?;
            let name: String = row.get("name")?;
            let bw_flag: i32 = row.get("bw_flag")?;
            ids.push((db_sid, name, bw_flag));
        }

        Ok(ids)
    }

    pub fn load_poi_by_sid(&self, sid: &str) -> Result<Option<CfPoi>> {
        let con = self.client.lock().unwrap();

        let sql = "select * from cf_poi where poi_sid = ?";
        let v = con.query_row(sql, params![sid], |row| CfPoi::scan(row)).optional()?;
        Ok(v)
    }

    pub fn upate_facetrack_for_judge(&self, po: &CfFacetrack) -> Result<usize> {
        let con = self.client.lock().unwrap();

        let sql = "update cf_facetrack set matched = ?, judged = ?, alarmed = ?, most_person = ?, most_score = ?, gmt_modified = ? where ft_sid = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![po.matched,po.judged,po.alarmed,po.most_person,po.most_score,po.gmt_modified,po.ft_sid])?;
        Ok(affect)
    }


    // ----------------------------

    pub fn save_cartrack(&self, po: &CfCartrack) -> Result<i64> {
        let mut guard = self.client.lock().unwrap();
        po.insert(&mut guard)
    }

    pub fn upate_cartrack_for_append(&self, po: &CfCartrack) -> Result<usize> {
        let con = self.client.lock().unwrap();

        let sql = "update cf_cartrack set img_ids = ?, plate_judged = ?, vehicle_judged = ?, move_direct = ?, car_direct = ?, plate_content = ?, plate_confidence = ?, plate_type = ?, car_color = ?, car_brand = ?, car_top_series = ?, car_series = ?, car_top_type = ?, car_mid_type = ?, lane_num = ?, gmt_modified = ? where sid = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![po.img_ids,po.plate_judged,po.vehicle_judged,po.move_direct,po.car_direct,po.plate_content, po.plate_confidence, po.plate_type,po.car_color,po.car_brand,po.car_top_series,po.car_series,po.car_top_type,po.car_mid_type,po.lane_num,po.gmt_modified,po.sid])?;
        Ok(affect)
    }

    pub fn load_coi_by_plate(&self, plate: &str) -> Result<Option<CfCoi>> {
        let con = self.client.lock().unwrap();

        let sql = "select * from cf_coi where plate_content = ?";
        let v = con.query_row(sql, params![plate], |row| CfCoi::scan(row)).optional()?;
        Ok(v)
    }

    pub fn load_coi_groups(&self) -> Result<Vec<(String, String, i32)>> {
        let con = self.client.lock().unwrap();

        let sql = "select sid,name,bw_flag from cf_coi_group";
        let mut stmt = con.prepare(sql)?;
        let mut rows = stmt.query(NO_PARAMS)?;

        let mut ids = Vec::new();
        while let Some(row) = rows.next()? {
            let sid: String = row.get("sid")?;
            let name: String = row.get("name")?;
            let bw_flag: i32 = row.get("bw_flag")?;
            ids.push((sid, name, bw_flag));
        }

        Ok(ids)
    }

    pub fn upate_cartrack_for_judge(&self, po: &CfCartrack) -> Result<usize> {
        let con = self.client.lock().unwrap();

        let sql = "update cf_cartrack set alarmed = ?, most_coi = ?, gmt_modified = ? where sid = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![po.alarmed,po.most_coi,po.gmt_modified,po.sid])?;
        Ok(affect)
    }

    pub fn get_facetrack_count(&self) -> Result<Option<i64>> {
        let sql = "select count(*) from cf_facetrack";
        let con = self.client.lock().unwrap();
        let mut stmt = con.prepare(sql)?;
        let v = stmt.query_row(NO_PARAMS, |x| x.get(0)).optional()?;
        Ok(v)
    }

    pub fn get_facetrack_alarm_count(&self) -> Result<Option<i64>> {
        let sql = "select count(*) from cf_facetrack where alarmed = 1";
        let con = self.client.lock().unwrap();
        let mut stmt = con.prepare(sql)?;
        let v = stmt.query_row(NO_PARAMS, |x| x.get(0)).optional()?;
        Ok(v)
    }

    pub fn get_cartrack_count(&self) -> Result<Option<i64>> {
        let sql = "select count(*) from cf_cartrack";
        let con = self.client.lock().unwrap();
        let mut stmt = con.prepare(sql)?;
        let v = stmt.query_row(NO_PARAMS, |x| x.get(0)).optional()?;
        Ok(v)
    }

    pub fn get_cartrack_alarm_count(&self) -> Result<Option<i64>> {
        let sql = "select count(*) from cf_cartrack where alarmed = 1";
        let con = self.client.lock().unwrap();
        let mut stmt = con.prepare(sql)?;
        let v = stmt.query_row(NO_PARAMS, |x| x.get(0)).optional()?;
        Ok(v)
    }

    pub fn load_latest_facetrack_list(&self, limit: i64) -> Result<Vec<CfFacetrack>> {
        let con = self.client.lock().unwrap();

        let sql = "select * from cf_facetrack order by id desc limit ?";
        let mut stmt = con.prepare(sql)?;
        let mut rows = stmt.query(params![limit])?;

        let mut list = Vec::new();
        while let Some(row) = rows.next()? {
            let po = CfFacetrack::scan(row)?;
            list.push(po);
        }
        Ok(list)
    }

    pub fn load_latest_cartrack_list(&self, limit: i64) -> Result<Vec<CfCartrack>> {
        let con = self.client.lock().unwrap();

        let sql = "select * from cf_cartrack order by id desc limit ?";
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
