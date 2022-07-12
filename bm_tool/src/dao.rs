use std::sync::Mutex;
use cffc_base::db::dbop;
use rusqlite::{NO_PARAMS};


pub struct AppDao {
    pub conn: Mutex<rusqlite::Connection>,
}


impl AppDao {
    pub fn new(conn: rusqlite::Connection) -> Self {
        AppDao {
            conn: Mutex::new(conn),
        }
    }

    pub fn delete_table(&self, table: &str) -> Result<usize, dbop::Error> {
        let conn = self.conn.lock().unwrap();
        let sql = format!("delete from {}", table);
        let affect = conn.execute(sql.as_str(), NO_PARAMS)?;
        Ok(affect)
    }

    pub fn get_dbs(&self) -> Result<Vec<(String, i64)>, dbop::Error> {
        let conn = self.conn.lock().unwrap();
        let sql = "select db_sid,capacity from cf_dfdb";

        let mut stmt = conn.prepare(sql)?;
        let mut rows = stmt.query(NO_PARAMS)?;

        let mut dbs = Vec::new();
        while let Some(row) = rows.next()? {
            let sid = row.get("db_sid")?;
            let capacity: i64 = row.get("capacity")?;
            dbs.push((sid, capacity));
        }

        Ok(dbs)
    }
}