use cffc_base::util::logger::InitLoggerErr;

#[derive(Debug)]
pub enum AppError {
    IO(String),
    LOG(String),
    JSON(String),
    NET(String),
    BMAPI(String),
    DB(String),
    COMMON(String),
}

pub type AppResult<T> = std::result::Result<T, AppError>;

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::IO(format!("{:?}", e))
    }
}


impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::JSON(format!("{:?}", e))
    }
}

impl From<InitLoggerErr> for AppError {
    fn from(_: InitLoggerErr) -> Self {
        AppError::LOG("InitLoggerErr".to_string())
    }
}