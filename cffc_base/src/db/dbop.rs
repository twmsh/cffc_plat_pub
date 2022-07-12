use std::{io, result};

//-------------------------------------
pub type Result<T, E = Error> = result::Result<T, E>;

pub trait DbOp<T> {
    type Conn;
    fn insert(&self, con: &mut Self::Conn) -> Result<i64>;
    fn update(&self, con: &mut Self::Conn) -> Result<usize>;
    fn delete(id: i64, con: &mut Self::Conn) -> Result<usize>;
    fn load(id: i64, con: &mut Self::Conn) -> Result<Option<T>>;
}

//-------------------------------------
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Sql(String),
    Common(String),
}

impl From<String> for Error {
    fn from(e: String) -> Error {
        Error::Sql(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

//--------------- rusqlite ----------------------
impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Error {
        Error::Sql(e.to_string())
    }
}

//--------------- mysql ----------------------
// impl From<mysql::error::Error> for Error {
//     fn from(e: mysql::error::Error) -> Error {
//         Error::Sql(e.to_string())
//     }
// }
