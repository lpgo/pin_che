use std::{fmt, result, error};
use bson;
use mongodb;
use db;
use entity;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use mongodb::db::{Database, ThreadedDatabase};
use rocket::http::Status;

pub type Result<T> = result::Result<T, ServiceError>;

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
    BsonEncoderError(bson::EncoderError),
    MongodbError(mongodb::Error),
    BsonOidError(bson::oid::Error),
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
            ServiceError::BsonEncoderError(ref e) => e.fmt(f),
            ServiceError::MongodbError(ref e) => e.fmt(f),
            ServiceError::BsonOidError(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for ServiceError {
    fn description(&self) -> &str {
        "pinche service error"
    }
}

pub struct Service {
    conn: db::DbConn,
    cache: db::CacheConn,
}

impl<'a, 'r> FromRequest<'a, 'r> for Service {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Service, ()> {
        let pool = request.guard::<State<db::Pool>>()?;
        let database = request.guard::<State<Database>>()?;
        match pool.get() {
            Ok(con) => {
                let service = Service {
                    conn: db::DbConn(database.clone()),
                    cache: db::CacheConn(con),
                };
                Outcome::Success(service)
            }
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

impl Service {
    pub fn add_user(&self, user: entity::User) -> Result<()> {
        println!("{:?}", user);
        self.conn.add(user).map(|_| ())
    }

    pub fn publish_trip(&self, trip: entity::Trip) -> Result<()> {
        Ok(())
    }

    pub fn get_tel(&self, openid: &str) -> Result<String> {
        Ok(openid.to_owned())
    }
}