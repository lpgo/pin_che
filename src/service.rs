use std::fmt;
use std::error;

#[derive(Debug)]
pub enum ServiceError {
    String(String),
    DontHaveEnoughSeats, //没有足够的座位
    NoCache(String),
    UserBusy(String, String),
    NoLogin,
    NoAuth,
    TimeOut, //距离出发时间不足半小时
    NotCount, //没有足够的使用次数
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ServiceError::NoLogin => write!(f, "you are not  login!"),
            ServiceError::NoAuth => write!(f, "you are not auth!"),
            ServiceError::NotCount => write!(f, "you are not enough count!"),
            ServiceError::TimeOut => {
                write!(f, "for trip start have not half hours,you can not refund!")
            }
            ServiceError::DontHaveEnoughSeats => write!(f, "this trip have not enough seats!"),
            ServiceError::String(ref s) => write!(f, "{}", s),
            ServiceError::NoCache(ref s) => write!(f, "{} no cache", s),
            ServiceError::UserBusy(ref s, ref m) => write!(f, "{} is busy,because {}", s, m),
        }
    }
}

impl error::Error for ServiceError {
    fn description(&self) -> &str {
        "pinche service error"
    }
}