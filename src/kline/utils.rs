use std::{error, fmt};

use chrono::{TimeZone, Utc};

/// 根据输入的时间戳(精确到秒)timestamp返回对应的日期时间datetime
pub fn timestamp_to_datetime(timestamp: f64) -> String {
    let secs = timestamp as i64;
    match Utc.timestamp_opt(secs, 0) {
        chrono::LocalResult::Single(datetime) => datetime.to_string(),
        _ => String::new(),
    }
}

/// 将日期字符串转换为时间戳
pub fn datetime_to_timestamp(datetime: &str) -> f64 {
    let date_time = Utc
        .datetime_from_str(datetime, "%Y-%m-%d %H:%M")
        .expect("日期字符解析失败");
    let timestamp = date_time.timestamp() as f64;
    timestamp
}

/// 错误类型
#[derive(Debug)]
pub enum CustomError {
    Http(reqwest::Error),
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CustomError::Http(err) => write!(f, "Http error: {}", err),
        }
    }
}

impl error::Error for CustomError {
    fn description(&self) -> &str {
        match self {
            CustomError::Http(err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match self {
            CustomError::Http(err) => Some(err),
        }
    }
}

impl From<reqwest::Error> for CustomError {
    fn from(err: reqwest::Error) -> Self {
        CustomError::Http(err)
    }
}
