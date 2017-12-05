use std::{self,fmt, result, error, convert, option};
use std::io::Cursor;
use bson;
use mongodb;
use db;
use entity;
use redis;
use serde_redis;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use rocket::response::{self, Response, Responder};
use rocket::http::ContentType;
use mongodb::db::Database;
use rocket::http::Status;
use hyper;


pub type Result<T> = result::Result<T, ServiceError>;

#[derive(Debug)]
pub enum ServiceError {
    String(String),
    DontHaveEnoughSeats, //没有足够的座位
    NoAuth,
    TimeOut, //距离出发时间不足半小时
    NotCount, //没有足够的使用次数
    BsonEncoderError(bson::EncoderError),
    BsonDecoderError(bson::DecoderError),
    MongodbError(mongodb::Error),
    BsonOidError(bson::oid::Error),
    NoneError(option::NoneError),
    RedisError(redis::RedisError),
    RedisDecodeError(serde_redis::decode::Error),
    StdIoError(std::io::Error),
    HyperUriError(hyper::error::UriError),
    HyperError(hyper::Error),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ServiceError::NoAuth => write!(f, "you are not auth!"),
            ServiceError::NotCount => write!(f, "you are not enough count!"),
            ServiceError::TimeOut => {
                write!(f, "for trip start have not half hours,you can not refund!")
            }
            ServiceError::DontHaveEnoughSeats => write!(f, "this trip have not enough seats!"),
            ServiceError::String(ref s) => write!(f, "{}", s),
            ServiceError::BsonEncoderError(ref e) => e.fmt(f),
            ServiceError::BsonDecoderError(ref e) => e.fmt(f),
            ServiceError::MongodbError(ref e) => e.fmt(f),
            ServiceError::BsonOidError(ref e) => e.fmt(f),
            ServiceError::NoneError(ref e) => write!(f, "{:?}", e),
            ServiceError::RedisError(ref e) => e.fmt(f),
            ServiceError::RedisDecodeError(ref e) => e.fmt(f),
            ServiceError::StdIoError(ref e) => e.fmt(f),
            ServiceError::HyperUriError(ref e) => e.fmt(f),
            ServiceError::HyperError(ref e) => e.fmt(f),
        }
    }
}

impl<'r> Responder<'r> for ServiceError {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        let mut builder = Response::build();
        builder.header(ContentType::JSON);
        match self {
            ServiceError::NoAuth => {
                builder.status(Status::Unauthorized).sized_body(
                    Cursor::new(
                        r#"{"status": "error", "reason": "Unauthorized, please login"}"#,
                    ),
                );
            }
            ServiceError::NotCount => {
                builder.status(Status::NotAcceptable).sized_body(
                    Cursor::new(
                        r#"{"status": "error", "reason": "you are not enough count!"}"#,
                    ),
                );
            },
            ServiceError::TimeOut => {
                builder.status(Status::NotAcceptable).sized_body(
                    Cursor::new(
                        r#"{"status": "error", "reason": "for trip start have not half hours,you can not refund!"}"#,
                    ),
                );
            },
            ServiceError::DontHaveEnoughSeats => {
                builder.status(Status::NotAcceptable).sized_body(
                    Cursor::new(
                        r#"{"status": "error", "reason": "this trip have not enough seats!"}"#,
                    ),
                );
            },
            ServiceError::String(ref s) => {
                builder.status(Status::BadRequest).sized_body(
                    Cursor::new(
                        format!("{{\"status\":\"error\",\"reason\":\"{}\"}}",s),
                    ),
                );
            },
            ServiceError::BsonEncoderError(ref e) => {
                builder.status(Status::UnprocessableEntity).sized_body(
                    Cursor::new(
                        format!("{{\"status\":\"error\",\"reason\":\"{:?}\"}}",e),
                    ),
                );
            },
            ServiceError::BsonDecoderError(ref e) => {
                builder.status(Status::UnprocessableEntity).sized_body(
                    Cursor::new(
                        format!("{{\"status\":\"error\",\"reason\":\"{:?}\"}}",e),
                    ),
                );
            },
            ServiceError::MongodbError(ref e) => {
                builder.status(Status::UnprocessableEntity).sized_body(
                    Cursor::new(
                        format!("{{\"status\":\"error\",\"reason\":\"{:?}\"}}",e),
                    ),
                );
            },
            ServiceError::BsonOidError(ref e) => {
                builder.status(Status::UnprocessableEntity).sized_body(
                    Cursor::new(
                        format!("{{\"status\":\"error\",\"reason\":\"{:?}\"}}",e),
                    ),
                );
            },
            ServiceError::NoneError(ref e) => {
                builder.status(Status::UnprocessableEntity).sized_body(
                    Cursor::new(
                        format!("{{\"status\":\"error\",\"reason\":\"{:?}\"}}",e),
                    ),
                );
            },
            ServiceError::RedisError(ref e) => {
                builder.status(Status::UnprocessableEntity).sized_body(
                    Cursor::new(
                        format!("{{\"status\":\"error\",\"reason\":\"{:?}\"}}",e),
                    ),
                );
            },
            ServiceError::RedisDecodeError(ref e) => {
                builder.status(Status::UnprocessableEntity).sized_body(
                    Cursor::new(
                        format!("{{\"status\":\"error\",\"reason\":\"{:?}\"}}",e),
                    ),
                );
            },
            ServiceError::StdIoError(ref e) => {
                builder.status(Status::UnprocessableEntity).sized_body(
                    Cursor::new(
                        format!("{{\"status\":\"error\",\"reason\":\"{:?}\"}}",e),
                    ),
                );
            },
            ServiceError::HyperUriError(ref e) => {
                builder.status(Status::UnprocessableEntity).sized_body(
                    Cursor::new(
                        format!("{{\"status\":\"error\",\"reason\":\"{:?}\"}}",e),
                    ),
                );
            },
            ServiceError::HyperError(ref e) => {
                builder.status(Status::UnprocessableEntity).sized_body(
                    Cursor::new(
                        format!("{{\"status\":\"error\",\"reason\":\"{:?}\"}}",e),
                    ),
                );
            },
        }
        builder.ok()
    }
}

impl error::Error for ServiceError {
    fn description(&self) -> &str {
        match *self {
            ServiceError::NoAuth => "you are not auth!",
            ServiceError::NotCount => "you are not enough count!",
            ServiceError::TimeOut => "for trip start have not half hours,you can not refund!",
            ServiceError::DontHaveEnoughSeats => "this trip have not enough seats!",
            ServiceError::String(ref s) => s.as_str(),
            ServiceError::BsonEncoderError(ref e) => e.description(),
            ServiceError::BsonDecoderError(ref e) => e.description(),
            ServiceError::MongodbError(ref e) => e.description(),
            ServiceError::BsonOidError(ref e) => e.description(),
            ServiceError::NoneError(_) => "option is None",
            ServiceError::RedisError(ref e) => e.description(),
            ServiceError::RedisDecodeError(ref e) => e.description(),
            ServiceError::StdIoError(ref e) => e.description(),
            ServiceError::HyperUriError(ref e) => e.description(),
            ServiceError::HyperError(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ServiceError::BsonEncoderError(ref e) => Some(e),
            ServiceError::BsonDecoderError(ref e) => Some(e),
            ServiceError::MongodbError(ref e) => Some(e),
            ServiceError::BsonOidError(ref e) => Some(e),
            ServiceError::RedisError(ref e) => Some(e),
            ServiceError::RedisDecodeError(ref e) => Some(e),
            ServiceError::StdIoError(ref e) => Some(e),
            ServiceError::HyperUriError(ref e) => Some(e),
            ServiceError::HyperError(ref e) => Some(e),
            _ => None,
        }
    }
}

impl convert::From<mongodb::Error> for ServiceError {
    fn from(err: mongodb::Error) -> Self {
        ServiceError::MongodbError(err)
    }
}

impl convert::From<option::NoneError> for ServiceError {
    fn from(err: option::NoneError) -> Self {
        ServiceError::NoneError(err)
    }
}

impl convert::From<redis::RedisError> for ServiceError {
    fn from(err: redis::RedisError) -> Self {
        ServiceError::RedisError(err)
    }
}

impl convert::From<std::io::Error> for ServiceError {
    fn from(err: std::io::Error) -> Self {
        ServiceError::StdIoError(err)
    }
}

impl convert::From<hyper::error::UriError> for ServiceError {
    fn from(err: hyper::error::UriError) -> Self {
        ServiceError::HyperUriError(err)
    }
}

impl convert::From<hyper::Error> for ServiceError {
    fn from(err: hyper::Error) -> Self {
        ServiceError::HyperError(err)
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
    pub fn new(conn:db::DbConn,cache:db::CacheConn) -> Self {
        Service{conn,cache}
    }

    pub fn add_user(&self, user: &entity::User) -> Result<()> {
        println!("{:?}", user);
        self.conn.add(user).map(|_| ())
    }

    pub fn publish_trip(&self, trip: &entity::Trip) -> Result<()> {
       self.cache.add_trip(trip)
    }

    pub fn get_tel(&self, openid: &str) -> Result<String> {
        self.conn.get_one::<entity::User>(openid).map(
            |user| user.tel,
        )
    }

    pub fn test(&self) {
        println!("service test");
    }
}