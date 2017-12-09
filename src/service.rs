use std::{self,fmt, result, error, convert, option};
use std::io::Cursor;
use bson;
use mongodb;
use db;
use entity;
use external;
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
    NoPay, //没有支付
    TripNotYours, //你不是车主
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
            ServiceError::TripNotYours => write!(f, "this trip is not yours, you can't discount"),
            ServiceError::NoPay => {
                write!(f, "you are not paid this trip")
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
                        r#"{"status": "ok", "reason": "Unauthorized, please login"}"#,
                    ),
                );
            }
            ServiceError::TripNotYours => {
                builder.status(Status::NotAcceptable).sized_body(
                    Cursor::new(
                        r#"{"status": "ok", "reason": "this trip is not yours, you can't discount"}"#,
                    ),
                );
            },
            ServiceError::NoPay => {
                builder.status(Status::NotAcceptable).sized_body(
                    Cursor::new(
                        r#"{"status": "ok", "reason": "you are not paid this trip"}"#,
                    ),
                );
            },
            ServiceError::DontHaveEnoughSeats => {
                builder.status(Status::NotAcceptable).sized_body(
                    Cursor::new(
                        r#"{"status": "ok", "reason": "this trip have not enough seats!"}"#,
                    ),
                );
            },
            ServiceError::String(ref s) => {
                builder.status(Status::BadRequest).sized_body(
                    Cursor::new(
                        format!("{{\"status\":\"ok\",\"reason\":\"{}\"}}",s),
                    ),
                );
            },
            ServiceError::BsonEncoderError(ref e) => {
                builder.status(Status::UnprocessableEntity).sized_body(
                    Cursor::new(
                        format!("{{\"status\":\"error\",\"reason\":\"BsonEncoderError: {:?}\"}}",e),
                    ),
                );
            },
            ServiceError::BsonDecoderError(ref e) => {
                builder.status(Status::UnprocessableEntity).sized_body(
                    Cursor::new(
                        format!("{{\"status\":\"error\",\"reason\":\"BsonDecoderError: {:?}\"}}",e),
                    ),
                );
            },
            ServiceError::MongodbError(ref e) => {
                builder.status(Status::UnprocessableEntity).sized_body(
                    Cursor::new(
                        format!("{{\"status\":\"error\",\"reason\":\"MongodbError: {:?}\"}}",e),
                    ),
                );
            },
            ServiceError::BsonOidError(ref e) => {
                builder.status(Status::UnprocessableEntity).sized_body(
                    Cursor::new(
                        format!("{{\"status\":\"error\",\"reason\":\"BsonOidError: {:?}\"}}",e),
                    ),
                );
            },
            ServiceError::NoneError(ref e) => {
                builder.status(Status::UnprocessableEntity).sized_body(
                    Cursor::new(
                        format!("{{\"status\":\"error\",\"reason\":\"NoneError: {:?}\"}}",e),
                    ),
                );
            },
            ServiceError::RedisError(ref e) => {
                builder.status(Status::UnprocessableEntity).sized_body(
                    Cursor::new(
                        format!("{{\"status\":\"error\",\"reason\":\"RedisError: {:?}\"}}",e),
                    ),
                );
            },
            ServiceError::RedisDecodeError(ref e) => {
                builder.status(Status::UnprocessableEntity).sized_body(
                    Cursor::new(
                        format!("{{\"status\":\"error\",\"reason\":\"RedisDecodeError: {:?}\"}}",e),
                    ),
                );
            },
            ServiceError::StdIoError(ref e) => {
                builder.status(Status::UnprocessableEntity).sized_body(
                    Cursor::new(
                        format!("{{\"status\":\"error\",\"reason\":\"StdIoError: {:?}\"}}",e),
                    ),
                );
            },
            ServiceError::HyperUriError(ref e) => {
                builder.status(Status::UnprocessableEntity).sized_body(
                    Cursor::new(
                        format!("{{\"status\":\"error\",\"reason\":\"HyperUriError: {:?}\"}}",e),
                    ),
                );
            },
            ServiceError::HyperError(ref e) => {
                builder.status(Status::UnprocessableEntity).sized_body(
                    Cursor::new(
                        format!("{{\"status\":\"error\",\"reason\":\"HyperError: {:?}\"}}",e),
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
            ServiceError::TripNotYours => "this trip is not yours",
            ServiceError::NoPay => "you are not paid this trip",
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

    pub fn publish_trip(&self, trip: &entity::Trip) -> Result<()> {
       self.cache.add_trip(trip)
    }

    pub fn apply_trip(&self, trip_id:String, openid:String, count:i64, tel:Option<String>) -> Result<entity::Order>{
        self.cache.get_object::<entity::Trip>(&trip_id)
            .map(|trip|entity::Order::new(trip,openid,count,tel))
            .and_then(|order| self.cache.add_order(&order).map(|_|order))
    }

    pub fn pay(&self,order_id:String,sign:String) -> Result<()> {
        println!("need check sign {}", sign); //?
        self.cache.pay_order(order_id)
    }

    pub fn discount(&self,order_id:String,fee:i64) -> Result<()> {
        let openid = "openid".to_owned();  //?
        self.cache.change_order_price(&order_id,&openid,-fee)
            .and_then(|transaction_id|external::refund(&order_id,&transaction_id,fee))
    }

    pub fn submit(&self, id:String) -> Result<()> {
        let trip_id = self.cache.submit_order(&id)?;
        let order:entity::Order = self.cache.get_object(&id)?;
        external::pay_to_client(&order.trip_owner, (order.price as f64 * 0.95) as i64)?;
        self.cache.check_trip_finish(&trip_id)
    }  

    pub fn test(&self) {
       
    }
}