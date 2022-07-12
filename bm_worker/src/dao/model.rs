// generate by sqlite_gen, don't edit it.
use cffc_base::db::dbop::{self, DbOp};
use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use rusqlite::{Connection, params, OptionalExtension};

//---------------------- BeUser ----------------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BeUser {
    pub id: i64,
    pub name: Option<String>,
    pub login_name: String,
    pub password: String,
    pub salt: String,
    pub token: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub service_flag: Option<i32>,
    pub ref_count: Option<i32>,
    pub last_login: Option<DateTime<Local>>,
    pub token_expire: Option<DateTime<Local>>,
    pub memo: Option<String>,
    pub gmt_create: DateTime<Local>,
    pub gmt_modified: DateTime<Local>,
}

impl BeUser {
    pub fn scan(row: &rusqlite::Row<'_>) -> rusqlite::Result<BeUser> {
        Ok(BeUser {
            id: row.get("id")?,
            name: row.get("name")?,
            login_name: row.get("login_name")?,
            password: row.get("password")?,
            salt: row.get("salt")?,
            token: row.get("token")?,
            phone: row.get("phone")?,
            email: row.get("email")?,
            service_flag: row.get("service_flag")?,
            ref_count: row.get("ref_count")?,
            last_login: row.get("last_login")?,
            token_expire: row.get("token_expire")?,
            memo: row.get("memo")?,
            gmt_create: row.get("gmt_create")?,
            gmt_modified: row.get("gmt_modified")?,
        })
    }
}

impl DbOp<BeUser> for BeUser {
    type Conn = Connection;

    fn insert(&self, con: &mut Self::Conn) -> Result<i64, dbop::Error> {
        let sql = "insert into be_user(name,login_name,password,salt,token,phone,email,service_flag,ref_count,last_login,token_expire,memo,gmt_create,gmt_modified) values(?,?,?,?,?,?,?,?,?,?,?,?,?,?)";
        let mut stmt = con.prepare(sql)?;
        let _affect = stmt.execute(params![self.name,self.login_name,self.password,self.salt,self.token,self.phone,self.email,self.service_flag,self.ref_count,self.last_login,self.token_expire,self.memo,self.gmt_create,self.gmt_modified])?;

        let id = con.last_insert_rowid();
        Ok(id)
    }

    fn update(&self, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "update be_user set name = ?, login_name = ?, password = ?, salt = ?, token = ?, phone = ?, email = ?, service_flag = ?, ref_count = ?, last_login = ?, token_expire = ?, memo = ?, gmt_create = ?, gmt_modified = ? where id = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![self.name,self.login_name,self.password,self.salt,self.token,self.phone,self.email,self.service_flag,self.ref_count,self.last_login,self.token_expire,self.memo,self.gmt_create,self.gmt_modified,self.id])?;
        Ok(affect)
    }

    fn delete(id: i64, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "delete from be_user where id = ?";
        let affect = con.execute(sql, params![id])?;
        Ok(affect)
    }

    fn load(id: i64, con: &mut Self::Conn) -> Result<Option<BeUser>, dbop::Error> {
        let sql = "select * from be_user where id = ?";
        let v = con.query_row(sql, params![id], |row| BeUser::scan(row)).optional()?;
        Ok(v)
    }
}

//---------------------- CfDfnode ----------------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CfDfnode {
    pub id: i64,
    pub node_sid: String,
    pub name: String,
    pub ip: String,
    pub url: String,
    pub node_type: i32,
    pub sort_num: Option<i32>,
    pub gmt_create: DateTime<Local>,
    pub gmt_modified: DateTime<Local>,
}

impl CfDfnode {
    pub fn scan(row: &rusqlite::Row<'_>) -> rusqlite::Result<CfDfnode> {
        Ok(CfDfnode {
            id: row.get("id")?,
            node_sid: row.get("node_sid")?,
            name: row.get("name")?,
            ip: row.get("ip")?,
            url: row.get("url")?,
            node_type: row.get("node_type")?,
            sort_num: row.get("sort_num")?,
            gmt_create: row.get("gmt_create")?,
            gmt_modified: row.get("gmt_modified")?,
        })
    }
}

impl DbOp<CfDfnode> for CfDfnode {
    type Conn = Connection;

    fn insert(&self, con: &mut Self::Conn) -> Result<i64, dbop::Error> {
        let sql = "insert into cf_dfnode(node_sid,name,ip,url,node_type,sort_num,gmt_create,gmt_modified) values(?,?,?,?,?,?,?,?)";
        let mut stmt = con.prepare(sql)?;
        let _affect = stmt.execute(params![self.node_sid,self.name,self.ip,self.url,self.node_type,self.sort_num,self.gmt_create,self.gmt_modified])?;

        let id = con.last_insert_rowid();
        Ok(id)
    }

    fn update(&self, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "update cf_dfnode set node_sid = ?, name = ?, ip = ?, url = ?, node_type = ?, sort_num = ?, gmt_create = ?, gmt_modified = ? where id = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![self.node_sid,self.name,self.ip,self.url,self.node_type,self.sort_num,self.gmt_create,self.gmt_modified,self.id])?;
        Ok(affect)
    }

    fn delete(id: i64, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "delete from cf_dfnode where id = ?";
        let affect = con.execute(sql, params![id])?;
        Ok(affect)
    }

    fn load(id: i64, con: &mut Self::Conn) -> Result<Option<CfDfnode>, dbop::Error> {
        let sql = "select * from cf_dfnode where id = ?";
        let v = con.query_row(sql, params![id], |row| CfDfnode::scan(row)).optional()?;
        Ok(v)
    }
}

//---------------------- CfDfdb ----------------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CfDfdb {
    pub id: i64,
    pub db_sid: String,
    pub name: String,
    pub node_sid: String,
    pub capacity: i64,
    pub auto_match: i32,
    pub bw_flag: i32,
    pub fp_flag: i32,
    pub sort_num: Option<i32>,
    pub gmt_create: DateTime<Local>,
    pub gmt_modified: DateTime<Local>,
}

impl CfDfdb {
    pub fn scan(row: &rusqlite::Row<'_>) -> rusqlite::Result<CfDfdb> {
        Ok(CfDfdb {
            id: row.get("id")?,
            db_sid: row.get("db_sid")?,
            name: row.get("name")?,
            node_sid: row.get("node_sid")?,
            capacity: row.get("capacity")?,
            auto_match: row.get("auto_match")?,
            bw_flag: row.get("bw_flag")?,
            fp_flag: row.get("fp_flag")?,
            sort_num: row.get("sort_num")?,
            gmt_create: row.get("gmt_create")?,
            gmt_modified: row.get("gmt_modified")?,
        })
    }
}

impl DbOp<CfDfdb> for CfDfdb {
    type Conn = Connection;

    fn insert(&self, con: &mut Self::Conn) -> Result<i64, dbop::Error> {
        let sql = "insert into cf_dfdb(db_sid,name,node_sid,capacity,auto_match,bw_flag,fp_flag,sort_num,gmt_create,gmt_modified) values(?,?,?,?,?,?,?,?,?,?)";
        let mut stmt = con.prepare(sql)?;
        let _affect = stmt.execute(params![self.db_sid,self.name,self.node_sid,self.capacity,self.auto_match,self.bw_flag,self.fp_flag,self.sort_num,self.gmt_create,self.gmt_modified])?;

        let id = con.last_insert_rowid();
        Ok(id)
    }

    fn update(&self, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "update cf_dfdb set db_sid = ?, name = ?, node_sid = ?, capacity = ?, auto_match = ?, bw_flag = ?, fp_flag = ?, sort_num = ?, gmt_create = ?, gmt_modified = ? where id = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![self.db_sid,self.name,self.node_sid,self.capacity,self.auto_match,self.bw_flag,self.fp_flag,self.sort_num,self.gmt_create,self.gmt_modified,self.id])?;
        Ok(affect)
    }

    fn delete(id: i64, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "delete from cf_dfdb where id = ?";
        let affect = con.execute(sql, params![id])?;
        Ok(affect)
    }

    fn load(id: i64, con: &mut Self::Conn) -> Result<Option<CfDfdb>, dbop::Error> {
        let sql = "select * from cf_dfdb where id = ?";
        let v = con.query_row(sql, params![id], |row| CfDfdb::scan(row)).optional()?;
        Ok(v)
    }
}

//---------------------- CfDfsource ----------------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CfDfsource {
    pub id: i64,
    pub src_sid: String,
    pub name: String,
    pub node_sid: String,
    pub src_url: String,
    pub push_url: String,
    pub ip: String,
    pub src_state: i32,
    pub src_config: String,
    pub grab_type: i32,
    pub io_flag: i32,
    pub direction: i32,
    pub tp_id: Option<String>,
    pub upload_flag: i32,
    pub location_name: Option<String>,
    pub resolution_ratio: Option<String>,
    pub coordinate: Option<String>,
    pub sort_num: i32,
    pub trip_line: i64,
    pub rtcp_utc: i32,
    pub lane_desc: Option<String>,
    pub lane_count: i32,
    pub memo: Option<String>,
    pub gmt_create: DateTime<Local>,
    pub gmt_modified: DateTime<Local>,
}

impl CfDfsource {
    pub fn scan(row: &rusqlite::Row<'_>) -> rusqlite::Result<CfDfsource> {
        Ok(CfDfsource {
            id: row.get("id")?,
            src_sid: row.get("src_sid")?,
            name: row.get("name")?,
            node_sid: row.get("node_sid")?,
            src_url: row.get("src_url")?,
            push_url: row.get("push_url")?,
            ip: row.get("ip")?,
            src_state: row.get("src_state")?,
            src_config: row.get("src_config")?,
            grab_type: row.get("grab_type")?,
            io_flag: row.get("io_flag")?,
            direction: row.get("direction")?,
            tp_id: row.get("tp_id")?,
            upload_flag: row.get("upload_flag")?,
            location_name: row.get("location_name")?,
            resolution_ratio: row.get("resolution_ratio")?,
            coordinate: row.get("coordinate")?,
            sort_num: row.get("sort_num")?,
            trip_line: row.get("trip_line")?,
            rtcp_utc: row.get("rtcp_utc")?,
            lane_desc: row.get("lane_desc")?,
            lane_count: row.get("lane_count")?,
            memo: row.get("memo")?,
            gmt_create: row.get("gmt_create")?,
            gmt_modified: row.get("gmt_modified")?,
        })
    }
}

impl DbOp<CfDfsource> for CfDfsource {
    type Conn = Connection;

    fn insert(&self, con: &mut Self::Conn) -> Result<i64, dbop::Error> {
        let sql = "insert into cf_dfsource(src_sid,name,node_sid,src_url,push_url,ip,src_state,src_config,grab_type,io_flag,direction,tp_id,upload_flag,location_name,resolution_ratio,coordinate,sort_num,trip_line,rtcp_utc,lane_desc,lane_count,memo,gmt_create,gmt_modified) values(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)";
        let mut stmt = con.prepare(sql)?;
        let _affect = stmt.execute(params![self.src_sid,self.name,self.node_sid,self.src_url,self.push_url,self.ip,self.src_state,self.src_config,self.grab_type,self.io_flag,self.direction,self.tp_id,self.upload_flag,self.location_name,self.resolution_ratio,self.coordinate,self.sort_num,self.trip_line,self.rtcp_utc,self.lane_desc,self.lane_count,self.memo,self.gmt_create,self.gmt_modified])?;

        let id = con.last_insert_rowid();
        Ok(id)
    }

    fn update(&self, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "update cf_dfsource set src_sid = ?, name = ?, node_sid = ?, src_url = ?, push_url = ?, ip = ?, src_state = ?, src_config = ?, grab_type = ?, io_flag = ?, direction = ?, tp_id = ?, upload_flag = ?, location_name = ?, resolution_ratio = ?, coordinate = ?, sort_num = ?, trip_line = ?, rtcp_utc = ?, lane_desc = ?, lane_count = ?, memo = ?, gmt_create = ?, gmt_modified = ? where id = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![self.src_sid,self.name,self.node_sid,self.src_url,self.push_url,self.ip,self.src_state,self.src_config,self.grab_type,self.io_flag,self.direction,self.tp_id,self.upload_flag,self.location_name,self.resolution_ratio,self.coordinate,self.sort_num,self.trip_line,self.rtcp_utc,self.lane_desc,self.lane_count,self.memo,self.gmt_create,self.gmt_modified,self.id])?;
        Ok(affect)
    }

    fn delete(id: i64, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "delete from cf_dfsource where id = ?";
        let affect = con.execute(sql, params![id])?;
        Ok(affect)
    }

    fn load(id: i64, con: &mut Self::Conn) -> Result<Option<CfDfsource>, dbop::Error> {
        let sql = "select * from cf_dfsource where id = ?";
        let v = con.query_row(sql, params![id], |row| CfDfsource::scan(row)).optional()?;
        Ok(v)
    }
}

//---------------------- CfPoi ----------------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CfPoi {
    pub id: i64,
    pub poi_sid: String,
    pub db_sid: String,
    pub name: String,
    pub gender: Option<i32>,
    pub identity_card: Option<String>,
    pub threshold: i32,
    pub tp_id: Option<String>,
    pub feature_ids: String,
    pub cover: Option<i32>,
    pub tag: Option<String>,
    pub imp_tag: Option<String>,
    pub memo: Option<String>,
    pub flag: Option<i32>,
    pub gmt_create: DateTime<Local>,
    pub gmt_modified: DateTime<Local>,
}

impl CfPoi {
    pub fn scan(row: &rusqlite::Row<'_>) -> rusqlite::Result<CfPoi> {
        Ok(CfPoi {
            id: row.get("id")?,
            poi_sid: row.get("poi_sid")?,
            db_sid: row.get("db_sid")?,
            name: row.get("name")?,
            gender: row.get("gender")?,
            identity_card: row.get("identity_card")?,
            threshold: row.get("threshold")?,
            tp_id: row.get("tp_id")?,
            feature_ids: row.get("feature_ids")?,
            cover: row.get("cover")?,
            tag: row.get("tag")?,
            imp_tag: row.get("imp_tag")?,
            memo: row.get("memo")?,
            flag: row.get("flag")?,
            gmt_create: row.get("gmt_create")?,
            gmt_modified: row.get("gmt_modified")?,
        })
    }
}

impl DbOp<CfPoi> for CfPoi {
    type Conn = Connection;

    fn insert(&self, con: &mut Self::Conn) -> Result<i64, dbop::Error> {
        let sql = "insert into cf_poi(poi_sid,db_sid,name,gender,identity_card,threshold,tp_id,feature_ids,cover,tag,imp_tag,memo,flag,gmt_create,gmt_modified) values(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)";
        let mut stmt = con.prepare(sql)?;
        let _affect = stmt.execute(params![self.poi_sid,self.db_sid,self.name,self.gender,self.identity_card,self.threshold,self.tp_id,self.feature_ids,self.cover,self.tag,self.imp_tag,self.memo,self.flag,self.gmt_create,self.gmt_modified])?;

        let id = con.last_insert_rowid();
        Ok(id)
    }

    fn update(&self, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "update cf_poi set poi_sid = ?, db_sid = ?, name = ?, gender = ?, identity_card = ?, threshold = ?, tp_id = ?, feature_ids = ?, cover = ?, tag = ?, imp_tag = ?, memo = ?, flag = ?, gmt_create = ?, gmt_modified = ? where id = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![self.poi_sid,self.db_sid,self.name,self.gender,self.identity_card,self.threshold,self.tp_id,self.feature_ids,self.cover,self.tag,self.imp_tag,self.memo,self.flag,self.gmt_create,self.gmt_modified,self.id])?;
        Ok(affect)
    }

    fn delete(id: i64, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "delete from cf_poi where id = ?";
        let affect = con.execute(sql, params![id])?;
        Ok(affect)
    }

    fn load(id: i64, con: &mut Self::Conn) -> Result<Option<CfPoi>, dbop::Error> {
        let sql = "select * from cf_poi where id = ?";
        let v = con.query_row(sql, params![id], |row| CfPoi::scan(row)).optional()?;
        Ok(v)
    }
}

//---------------------- CfFacetrack ----------------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CfFacetrack {
    pub id: i64,
    pub ft_sid: String,
    pub src_sid: String,
    pub img_ids: String,
    pub matched: Option<i32>,
    pub judged: Option<i32>,
    pub alarmed: Option<i32>,
    pub most_person: Option<String>,
    pub most_score: Option<f64>,
    pub gender: Option<i32>,
    pub age: Option<i32>,
    pub glasses: Option<i32>,
    pub direction: Option<i32>,
    pub plane_score: Option<f64>,
    pub mask: Option<i32>,
    pub moustache: Option<i32>,
    pub hat: Option<i32>,
    pub tag: Option<String>,
    pub flag: i32,
    pub db_flag: Option<i32>,
    pub db_sid: Option<String>,
    pub feature_ids: Option<String>,
    pub obj_id: Option<String>,
    pub submit_id: Option<String>,
    pub submit_time: Option<DateTime<Local>>,
    pub capture_time: DateTime<Local>,
    pub gmt_create: DateTime<Local>,
    pub gmt_modified: DateTime<Local>,
}

impl CfFacetrack {
    pub fn scan(row: &rusqlite::Row<'_>) -> rusqlite::Result<CfFacetrack> {
        Ok(CfFacetrack {
            id: row.get("id")?,
            ft_sid: row.get("ft_sid")?,
            src_sid: row.get("src_sid")?,
            img_ids: row.get("img_ids")?,
            matched: row.get("matched")?,
            judged: row.get("judged")?,
            alarmed: row.get("alarmed")?,
            most_person: row.get("most_person")?,
            most_score: row.get("most_score")?,
            gender: row.get("gender")?,
            age: row.get("age")?,
            glasses: row.get("glasses")?,
            direction: row.get("direction")?,
            plane_score: row.get("plane_score")?,
            mask: row.get("mask")?,
            moustache: row.get("moustache")?,
            hat: row.get("hat")?,
            tag: row.get("tag")?,
            flag: row.get("flag")?,
            db_flag: row.get("db_flag")?,
            db_sid: row.get("db_sid")?,
            feature_ids: row.get("feature_ids")?,
            obj_id: row.get("obj_id")?,
            submit_id: row.get("submit_id")?,
            submit_time: row.get("submit_time")?,
            capture_time: row.get("capture_time")?,
            gmt_create: row.get("gmt_create")?,
            gmt_modified: row.get("gmt_modified")?,
        })
    }
}

impl DbOp<CfFacetrack> for CfFacetrack {
    type Conn = Connection;

    fn insert(&self, con: &mut Self::Conn) -> Result<i64, dbop::Error> {
        let sql = "insert into cf_facetrack(ft_sid,src_sid,img_ids,matched,judged,alarmed,most_person,most_score,gender,age,glasses,direction,plane_score,mask,moustache,hat,tag,flag,db_flag,db_sid,feature_ids,obj_id,submit_id,submit_time,capture_time,gmt_create,gmt_modified) values(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)";
        let mut stmt = con.prepare(sql)?;
        let _affect = stmt.execute(params![self.ft_sid,self.src_sid,self.img_ids,self.matched,self.judged,self.alarmed,self.most_person,self.most_score,self.gender,self.age,self.glasses,self.direction,self.plane_score,self.mask,self.moustache,self.hat,self.tag,self.flag,self.db_flag,self.db_sid,self.feature_ids,self.obj_id,self.submit_id,self.submit_time,self.capture_time,self.gmt_create,self.gmt_modified])?;

        let id = con.last_insert_rowid();
        Ok(id)
    }

    fn update(&self, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "update cf_facetrack set ft_sid = ?, src_sid = ?, img_ids = ?, matched = ?, judged = ?, alarmed = ?, most_person = ?, most_score = ?, gender = ?, age = ?, glasses = ?, direction = ?, plane_score = ?, mask = ?, moustache = ?, hat = ?, tag = ?, flag = ?, db_flag = ?, db_sid = ?, feature_ids = ?, obj_id = ?, submit_id = ?, submit_time = ?, capture_time = ?, gmt_create = ?, gmt_modified = ? where id = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![self.ft_sid,self.src_sid,self.img_ids,self.matched,self.judged,self.alarmed,self.most_person,self.most_score,self.gender,self.age,self.glasses,self.direction,self.plane_score,self.mask,self.moustache,self.hat,self.tag,self.flag,self.db_flag,self.db_sid,self.feature_ids,self.obj_id,self.submit_id,self.submit_time,self.capture_time,self.gmt_create,self.gmt_modified,self.id])?;
        Ok(affect)
    }

    fn delete(id: i64, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "delete from cf_facetrack where id = ?";
        let affect = con.execute(sql, params![id])?;
        Ok(affect)
    }

    fn load(id: i64, con: &mut Self::Conn) -> Result<Option<CfFacetrack>, dbop::Error> {
        let sql = "select * from cf_facetrack where id = ?";
        let v = con.query_row(sql, params![id], |row| CfFacetrack::scan(row)).optional()?;
        Ok(v)
    }
}

//---------------------- CfDictory ----------------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CfDictory {
    pub id: i64,
    pub group_label: String,
    pub group_key: String,
    pub item_label: String,
    pub item_key: String,
    pub item_value: Option<String>,
    pub sort_num: Option<i32>,
    pub memo: Option<String>,
    pub gmt_create: DateTime<Local>,
    pub gmt_modified: DateTime<Local>,
}

impl CfDictory {
    pub fn scan(row: &rusqlite::Row<'_>) -> rusqlite::Result<CfDictory> {
        Ok(CfDictory {
            id: row.get("id")?,
            group_label: row.get("group_label")?,
            group_key: row.get("group_key")?,
            item_label: row.get("item_label")?,
            item_key: row.get("item_key")?,
            item_value: row.get("item_value")?,
            sort_num: row.get("sort_num")?,
            memo: row.get("memo")?,
            gmt_create: row.get("gmt_create")?,
            gmt_modified: row.get("gmt_modified")?,
        })
    }
}

impl DbOp<CfDictory> for CfDictory {
    type Conn = Connection;

    fn insert(&self, con: &mut Self::Conn) -> Result<i64, dbop::Error> {
        let sql = "insert into cf_dictory(group_label,group_key,item_label,item_key,item_value,sort_num,memo,gmt_create,gmt_modified) values(?,?,?,?,?,?,?,?,?)";
        let mut stmt = con.prepare(sql)?;
        let _affect = stmt.execute(params![self.group_label,self.group_key,self.item_label,self.item_key,self.item_value,self.sort_num,self.memo,self.gmt_create,self.gmt_modified])?;

        let id = con.last_insert_rowid();
        Ok(id)
    }

    fn update(&self, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "update cf_dictory set group_label = ?, group_key = ?, item_label = ?, item_key = ?, item_value = ?, sort_num = ?, memo = ?, gmt_create = ?, gmt_modified = ? where id = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![self.group_label,self.group_key,self.item_label,self.item_key,self.item_value,self.sort_num,self.memo,self.gmt_create,self.gmt_modified,self.id])?;
        Ok(affect)
    }

    fn delete(id: i64, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "delete from cf_dictory where id = ?";
        let affect = con.execute(sql, params![id])?;
        Ok(affect)
    }

    fn load(id: i64, con: &mut Self::Conn) -> Result<Option<CfDictory>, dbop::Error> {
        let sql = "select * from cf_dictory where id = ?";
        let v = con.query_row(sql, params![id], |row| CfDictory::scan(row)).optional()?;
        Ok(v)
    }
}

//---------------------- CfDelpoi ----------------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CfDelpoi {
    pub id: i64,
    pub poi_id: i64,
    pub poi_sid: String,
    pub db_sid: String,
    pub name: String,
    pub tp_id: Option<String>,
    pub gmt_create: DateTime<Local>,
    pub gmt_modified: DateTime<Local>,
}

impl CfDelpoi {
    pub fn scan(row: &rusqlite::Row<'_>) -> rusqlite::Result<CfDelpoi> {
        Ok(CfDelpoi {
            id: row.get("id")?,
            poi_id: row.get("poi_id")?,
            poi_sid: row.get("poi_sid")?,
            db_sid: row.get("db_sid")?,
            name: row.get("name")?,
            tp_id: row.get("tp_id")?,
            gmt_create: row.get("gmt_create")?,
            gmt_modified: row.get("gmt_modified")?,
        })
    }
}

impl DbOp<CfDelpoi> for CfDelpoi {
    type Conn = Connection;

    fn insert(&self, con: &mut Self::Conn) -> Result<i64, dbop::Error> {
        let sql = "insert into cf_delpoi(poi_id,poi_sid,db_sid,name,tp_id,gmt_create,gmt_modified) values(?,?,?,?,?,?,?)";
        let mut stmt = con.prepare(sql)?;
        let _affect = stmt.execute(params![self.poi_id,self.poi_sid,self.db_sid,self.name,self.tp_id,self.gmt_create,self.gmt_modified])?;

        let id = con.last_insert_rowid();
        Ok(id)
    }

    fn update(&self, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "update cf_delpoi set poi_id = ?, poi_sid = ?, db_sid = ?, name = ?, tp_id = ?, gmt_create = ?, gmt_modified = ? where id = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![self.poi_id,self.poi_sid,self.db_sid,self.name,self.tp_id,self.gmt_create,self.gmt_modified,self.id])?;
        Ok(affect)
    }

    fn delete(id: i64, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "delete from cf_delpoi where id = ?";
        let affect = con.execute(sql, params![id])?;
        Ok(affect)
    }

    fn load(id: i64, con: &mut Self::Conn) -> Result<Option<CfDelpoi>, dbop::Error> {
        let sql = "select * from cf_delpoi where id = ?";
        let v = con.query_row(sql, params![id], |row| CfDelpoi::scan(row)).optional()?;
        Ok(v)
    }
}

//---------------------- CfCartrack ----------------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CfCartrack {
    pub id: i64,
    pub sid: String,
    pub src_sid: String,
    pub img_ids: String,
    pub alarmed: i32,
    pub most_coi: Option<String>,
    pub plate_judged: i32,
    pub vehicle_judged: i32,
    pub move_direct: i32,
    pub car_direct: Option<String>,
    pub plate_content: Option<String>,
    pub plate_confidence: Option<f64>,
    pub plate_type: Option<String>,
    pub car_color: Option<String>,
    pub car_brand: Option<String>,
    pub car_top_series: Option<String>,
    pub car_series: Option<String>,
    pub car_top_type: Option<String>,
    pub car_mid_type: Option<String>,
    pub tag: Option<String>,
    pub flag: i32,
    pub obj_id: Option<String>,
    pub submit_id: Option<String>,
    pub submit_time: Option<DateTime<Local>>,
    pub is_realtime: i32,
    pub capture_time: DateTime<Local>,
    pub capture_ts: i64,
    pub capture_pts: i64,
    pub lane_num: i32,
    pub gmt_create: DateTime<Local>,
    pub gmt_modified: DateTime<Local>,
}

impl CfCartrack {
    pub fn scan(row: &rusqlite::Row<'_>) -> rusqlite::Result<CfCartrack> {
        Ok(CfCartrack {
            id: row.get("id")?,
            sid: row.get("sid")?,
            src_sid: row.get("src_sid")?,
            img_ids: row.get("img_ids")?,
            alarmed: row.get("alarmed")?,
            most_coi: row.get("most_coi")?,
            plate_judged: row.get("plate_judged")?,
            vehicle_judged: row.get("vehicle_judged")?,
            move_direct: row.get("move_direct")?,
            car_direct: row.get("car_direct")?,
            plate_content: row.get("plate_content")?,
            plate_confidence: row.get("plate_confidence")?,
            plate_type: row.get("plate_type")?,
            car_color: row.get("car_color")?,
            car_brand: row.get("car_brand")?,
            car_top_series: row.get("car_top_series")?,
            car_series: row.get("car_series")?,
            car_top_type: row.get("car_top_type")?,
            car_mid_type: row.get("car_mid_type")?,
            tag: row.get("tag")?,
            flag: row.get("flag")?,
            obj_id: row.get("obj_id")?,
            submit_id: row.get("submit_id")?,
            submit_time: row.get("submit_time")?,
            is_realtime: row.get("is_realtime")?,
            capture_time: row.get("capture_time")?,
            capture_ts: row.get("capture_ts")?,
            capture_pts: row.get("capture_pts")?,
            lane_num: row.get("lane_num")?,
            gmt_create: row.get("gmt_create")?,
            gmt_modified: row.get("gmt_modified")?,
        })
    }
}

impl DbOp<CfCartrack> for CfCartrack {
    type Conn = Connection;

    fn insert(&self, con: &mut Self::Conn) -> Result<i64, dbop::Error> {
        let sql = "insert into cf_cartrack(sid,src_sid,img_ids,alarmed,most_coi,plate_judged,vehicle_judged,move_direct,car_direct,plate_content,plate_confidence,plate_type,car_color,car_brand,car_top_series,car_series,car_top_type,car_mid_type,tag,flag,obj_id,submit_id,submit_time,is_realtime,capture_time,capture_ts,capture_pts,lane_num,gmt_create,gmt_modified) values(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)";
        let mut stmt = con.prepare(sql)?;
        let _affect = stmt.execute(params![self.sid,self.src_sid,self.img_ids,self.alarmed,self.most_coi,self.plate_judged,self.vehicle_judged,self.move_direct,self.car_direct,self.plate_content,self.plate_confidence,self.plate_type,self.car_color,self.car_brand,self.car_top_series,self.car_series,self.car_top_type,self.car_mid_type,self.tag,self.flag,self.obj_id,self.submit_id,self.submit_time,self.is_realtime,self.capture_time,self.capture_ts,self.capture_pts,self.lane_num,self.gmt_create,self.gmt_modified])?;

        let id = con.last_insert_rowid();
        Ok(id)
    }

    fn update(&self, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "update cf_cartrack set sid = ?, src_sid = ?, img_ids = ?, alarmed = ?, most_coi = ?, plate_judged = ?, vehicle_judged = ?, move_direct = ?, car_direct = ?, plate_content = ?, plate_confidence = ?, plate_type = ?, car_color = ?, car_brand = ?, car_top_series = ?, car_series = ?, car_top_type = ?, car_mid_type = ?, tag = ?, flag = ?, obj_id = ?, submit_id = ?, submit_time = ?, is_realtime = ?, capture_time = ?, capture_ts = ?, capture_pts = ?, lane_num = ?, gmt_create = ?, gmt_modified = ? where id = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![self.sid,self.src_sid,self.img_ids,self.alarmed,self.most_coi,self.plate_judged,self.vehicle_judged,self.move_direct,self.car_direct,self.plate_content,self.plate_confidence,self.plate_type,self.car_color,self.car_brand,self.car_top_series,self.car_series,self.car_top_type,self.car_mid_type,self.tag,self.flag,self.obj_id,self.submit_id,self.submit_time,self.is_realtime,self.capture_time,self.capture_ts,self.capture_pts,self.lane_num,self.gmt_create,self.gmt_modified,self.id])?;
        Ok(affect)
    }

    fn delete(id: i64, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "delete from cf_cartrack where id = ?";
        let affect = con.execute(sql, params![id])?;
        Ok(affect)
    }

    fn load(id: i64, con: &mut Self::Conn) -> Result<Option<CfCartrack>, dbop::Error> {
        let sql = "select * from cf_cartrack where id = ?";
        let v = con.query_row(sql, params![id], |row| CfCartrack::scan(row)).optional()?;
        Ok(v)
    }
}

//---------------------- CfCoi ----------------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CfCoi {
    pub id: i64,
    pub sid: String,
    pub group_sid: String,
    pub plate_content: String,
    pub plate_type: Option<String>,
    pub car_brand: Option<String>,
    pub car_series: Option<String>,
    pub car_size: Option<String>,
    pub car_type: Option<String>,
    pub owner_name: Option<String>,
    pub owner_idcard: Option<String>,
    pub owner_phone: Option<String>,
    pub owner_address: Option<String>,
    pub flag: i32,
    pub tag: Option<String>,
    pub imp_tag: Option<String>,
    pub memo: Option<String>,
    pub gmt_create: DateTime<Local>,
    pub gmt_modified: DateTime<Local>,
}

impl CfCoi {
    pub fn scan(row: &rusqlite::Row<'_>) -> rusqlite::Result<CfCoi> {
        Ok(CfCoi {
            id: row.get("id")?,
            sid: row.get("sid")?,
            group_sid: row.get("group_sid")?,
            plate_content: row.get("plate_content")?,
            plate_type: row.get("plate_type")?,
            car_brand: row.get("car_brand")?,
            car_series: row.get("car_series")?,
            car_size: row.get("car_size")?,
            car_type: row.get("car_type")?,
            owner_name: row.get("owner_name")?,
            owner_idcard: row.get("owner_idcard")?,
            owner_phone: row.get("owner_phone")?,
            owner_address: row.get("owner_address")?,
            flag: row.get("flag")?,
            tag: row.get("tag")?,
            imp_tag: row.get("imp_tag")?,
            memo: row.get("memo")?,
            gmt_create: row.get("gmt_create")?,
            gmt_modified: row.get("gmt_modified")?,
        })
    }
}

impl DbOp<CfCoi> for CfCoi {
    type Conn = Connection;

    fn insert(&self, con: &mut Self::Conn) -> Result<i64, dbop::Error> {
        let sql = "insert into cf_coi(sid,group_sid,plate_content,plate_type,car_brand,car_series,car_size,car_type,owner_name,owner_idcard,owner_phone,owner_address,flag,tag,imp_tag,memo,gmt_create,gmt_modified) values(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)";
        let mut stmt = con.prepare(sql)?;
        let _affect = stmt.execute(params![self.sid,self.group_sid,self.plate_content,self.plate_type,self.car_brand,self.car_series,self.car_size,self.car_type,self.owner_name,self.owner_idcard,self.owner_phone,self.owner_address,self.flag,self.tag,self.imp_tag,self.memo,self.gmt_create,self.gmt_modified])?;

        let id = con.last_insert_rowid();
        Ok(id)
    }

    fn update(&self, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "update cf_coi set sid = ?, group_sid = ?, plate_content = ?, plate_type = ?, car_brand = ?, car_series = ?, car_size = ?, car_type = ?, owner_name = ?, owner_idcard = ?, owner_phone = ?, owner_address = ?, flag = ?, tag = ?, imp_tag = ?, memo = ?, gmt_create = ?, gmt_modified = ? where id = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![self.sid,self.group_sid,self.plate_content,self.plate_type,self.car_brand,self.car_series,self.car_size,self.car_type,self.owner_name,self.owner_idcard,self.owner_phone,self.owner_address,self.flag,self.tag,self.imp_tag,self.memo,self.gmt_create,self.gmt_modified,self.id])?;
        Ok(affect)
    }

    fn delete(id: i64, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "delete from cf_coi where id = ?";
        let affect = con.execute(sql, params![id])?;
        Ok(affect)
    }

    fn load(id: i64, con: &mut Self::Conn) -> Result<Option<CfCoi>, dbop::Error> {
        let sql = "select * from cf_coi where id = ?";
        let v = con.query_row(sql, params![id], |row| CfCoi::scan(row)).optional()?;
        Ok(v)
    }
}

//---------------------- CfCoiGroup ----------------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CfCoiGroup {
    pub id: i64,
    pub sid: String,
    pub name: String,
    pub bw_flag: i32,
    pub memo: Option<String>,
    pub gmt_create: DateTime<Local>,
    pub gmt_modified: DateTime<Local>,
}

impl CfCoiGroup {
    pub fn scan(row: &rusqlite::Row<'_>) -> rusqlite::Result<CfCoiGroup> {
        Ok(CfCoiGroup {
            id: row.get("id")?,
            sid: row.get("sid")?,
            name: row.get("name")?,
            bw_flag: row.get("bw_flag")?,
            memo: row.get("memo")?,
            gmt_create: row.get("gmt_create")?,
            gmt_modified: row.get("gmt_modified")?,
        })
    }
}

impl DbOp<CfCoiGroup> for CfCoiGroup {
    type Conn = Connection;

    fn insert(&self, con: &mut Self::Conn) -> Result<i64, dbop::Error> {
        let sql = "insert into cf_coi_group(sid,name,bw_flag,memo,gmt_create,gmt_modified) values(?,?,?,?,?,?)";
        let mut stmt = con.prepare(sql)?;
        let _affect = stmt.execute(params![self.sid,self.name,self.bw_flag,self.memo,self.gmt_create,self.gmt_modified])?;

        let id = con.last_insert_rowid();
        Ok(id)
    }

    fn update(&self, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "update cf_coi_group set sid = ?, name = ?, bw_flag = ?, memo = ?, gmt_create = ?, gmt_modified = ? where id = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![self.sid,self.name,self.bw_flag,self.memo,self.gmt_create,self.gmt_modified,self.id])?;
        Ok(affect)
    }

    fn delete(id: i64, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "delete from cf_coi_group where id = ?";
        let affect = con.execute(sql, params![id])?;
        Ok(affect)
    }

    fn load(id: i64, con: &mut Self::Conn) -> Result<Option<CfCoiGroup>, dbop::Error> {
        let sql = "select * from cf_coi_group where id = ?";
        let v = con.query_row(sql, params![id], |row| CfCoiGroup::scan(row)).optional()?;
        Ok(v)
    }
}

//---------------------- CfGate ----------------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CfGate {
    pub id: i64,
    pub src_sid: String,
    pub sid: String,
    pub name: String,
    pub uni_code: Option<String>,
    pub flag: Option<i32>,
    pub ac_config: String,
    pub ac_type: Option<i32>,
    pub sort_num: Option<i32>,
    pub memo: Option<String>,
    pub gmt_create: DateTime<Local>,
    pub gmt_modified: DateTime<Local>,
}

impl CfGate {
    pub fn scan(row: &rusqlite::Row<'_>) -> rusqlite::Result<CfGate> {
        Ok(CfGate {
            id: row.get("id")?,
            src_sid: row.get("src_sid")?,
            sid: row.get("sid")?,
            name: row.get("name")?,
            uni_code: row.get("uni_code")?,
            flag: row.get("flag")?,
            ac_config: row.get("ac_config")?,
            ac_type: row.get("ac_type")?,
            sort_num: row.get("sort_num")?,
            memo: row.get("memo")?,
            gmt_create: row.get("gmt_create")?,
            gmt_modified: row.get("gmt_modified")?,
        })
    }
}

impl DbOp<CfGate> for CfGate {
    type Conn = Connection;

    fn insert(&self, con: &mut Self::Conn) -> Result<i64, dbop::Error> {
        let sql = "insert into cf_gate(src_sid,sid,name,uni_code,flag,ac_config,ac_type,sort_num,memo,gmt_create,gmt_modified) values(?,?,?,?,?,?,?,?,?,?,?)";
        let mut stmt = con.prepare(sql)?;
        let _affect = stmt.execute(params![self.src_sid,self.sid,self.name,self.uni_code,self.flag,self.ac_config,self.ac_type,self.sort_num,self.memo,self.gmt_create,self.gmt_modified])?;

        let id = con.last_insert_rowid();
        Ok(id)
    }

    fn update(&self, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "update cf_gate set src_sid = ?, sid = ?, name = ?, uni_code = ?, flag = ?, ac_config = ?, ac_type = ?, sort_num = ?, memo = ?, gmt_create = ?, gmt_modified = ? where id = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![self.src_sid,self.sid,self.name,self.uni_code,self.flag,self.ac_config,self.ac_type,self.sort_num,self.memo,self.gmt_create,self.gmt_modified,self.id])?;
        Ok(affect)
    }

    fn delete(id: i64, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "delete from cf_gate where id = ?";
        let affect = con.execute(sql, params![id])?;
        Ok(affect)
    }

    fn load(id: i64, con: &mut Self::Conn) -> Result<Option<CfGate>, dbop::Error> {
        let sql = "select * from cf_gate where id = ?";
        let v = con.query_row(sql, params![id], |row| CfGate::scan(row)).optional()?;
        Ok(v)
    }
}

//---------------------- CfGatehistory ----------------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CfGatehistory {
    pub id: i64,
    pub ft_sid: String,
    pub src_sid: String,
    pub src_name: String,
    pub gate_sid: String,
    pub gate_name: String,
    pub poi_sid: String,
    pub poi_name: String,
    pub poi_idcard: Option<String>,
    pub gmt_create: DateTime<Local>,
    pub gmt_modified: DateTime<Local>,
}

impl CfGatehistory {
    pub fn scan(row: &rusqlite::Row<'_>) -> rusqlite::Result<CfGatehistory> {
        Ok(CfGatehistory {
            id: row.get("id")?,
            ft_sid: row.get("ft_sid")?,
            src_sid: row.get("src_sid")?,
            src_name: row.get("src_name")?,
            gate_sid: row.get("gate_sid")?,
            gate_name: row.get("gate_name")?,
            poi_sid: row.get("poi_sid")?,
            poi_name: row.get("poi_name")?,
            poi_idcard: row.get("poi_idcard")?,
            gmt_create: row.get("gmt_create")?,
            gmt_modified: row.get("gmt_modified")?,
        })
    }
}

impl DbOp<CfGatehistory> for CfGatehistory {
    type Conn = Connection;

    fn insert(&self, con: &mut Self::Conn) -> Result<i64, dbop::Error> {
        let sql = "insert into cf_gatehistory(ft_sid,src_sid,src_name,gate_sid,gate_name,poi_sid,poi_name,poi_idcard,gmt_create,gmt_modified) values(?,?,?,?,?,?,?,?,?,?)";
        let mut stmt = con.prepare(sql)?;
        let _affect = stmt.execute(params![self.ft_sid,self.src_sid,self.src_name,self.gate_sid,self.gate_name,self.poi_sid,self.poi_name,self.poi_idcard,self.gmt_create,self.gmt_modified])?;

        let id = con.last_insert_rowid();
        Ok(id)
    }

    fn update(&self, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "update cf_gatehistory set ft_sid = ?, src_sid = ?, src_name = ?, gate_sid = ?, gate_name = ?, poi_sid = ?, poi_name = ?, poi_idcard = ?, gmt_create = ?, gmt_modified = ? where id = ?";
        let mut stmt = con.prepare(sql)?;
        let affect = stmt.execute(params![self.ft_sid,self.src_sid,self.src_name,self.gate_sid,self.gate_name,self.poi_sid,self.poi_name,self.poi_idcard,self.gmt_create,self.gmt_modified,self.id])?;
        Ok(affect)
    }

    fn delete(id: i64, con: &mut Self::Conn) -> Result<usize, dbop::Error> {
        let sql = "delete from cf_gatehistory where id = ?";
        let affect = con.execute(sql, params![id])?;
        Ok(affect)
    }

    fn load(id: i64, con: &mut Self::Conn) -> Result<Option<CfGatehistory>, dbop::Error> {
        let sql = "select * from cf_gatehistory where id = ?";
        let v = con.query_row(sql, params![id], |row| CfGatehistory::scan(row)).optional()?;
        Ok(v)
    }
}

