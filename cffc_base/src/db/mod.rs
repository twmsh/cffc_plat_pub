pub mod dbop;

use std::sync::Mutex;
use std::ops::Deref;

pub struct SqliteClient {
    pub connection: Mutex<rusqlite::Connection>,
}

impl SqliteClient {
    pub fn new(con: rusqlite::Connection) -> Self {
        SqliteClient {
            connection: Mutex::new(con),
        }
    }
}

impl Deref for SqliteClient {
    type Target = Mutex<rusqlite::Connection>;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}