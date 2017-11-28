use mongodb::{Client, ThreadedClient};
use mongodb::db::{Database, ThreadedDatabase};
use std::ops::Deref;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use rocket::http::Status;
use r2d2;
use r2d2_redis::RedisConnectionManager;
use redis::Connection;
use setting;
use service::{ServiceError, Result};
use entity;
use serde::ser::Serialize;
use bson::{self, Document, Bson};
use bson::oid::ObjectId;


pub type Pool = r2d2::Pool<RedisConnectionManager>;

pub struct DbConn(pub Database);
pub struct CacheConn(pub r2d2::PooledConnection<RedisConnectionManager>);

//获取mongodb中的name
pub trait GetName {
    fn get_name() -> &'static str;
}

impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, ()> {
        let pool = request.guard::<State<Database>>()?;
        Outcome::Success(DbConn(pool.clone()))
    }
}

impl Deref for DbConn {
    type Target = Database;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn init_db_conn() -> Database {
    Client::connect(
        &setting::get_str("app.dburl"),
        setting::get_int64("app.dbport") as u16,
    ).expect("can't connect db")
        .db("test")
}




pub fn init_redis() -> Pool {
    let config = Default::default();
    let manager = RedisConnectionManager::new(setting::get_str("app.redis").as_str())
        .expect("can't open redis!!");
    r2d2::Pool::new(config, manager).expect("can't pooled redis conection!!")
}

impl<'a, 'r> FromRequest<'a, 'r> for CacheConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<CacheConn, ()> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(CacheConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

impl Deref for CacheConn {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl GetName for entity::User {
    fn get_name() -> &'static str {
        "User"
    }
}

impl GetName for entity::Order {
    fn get_name() -> &'static str {
        "Order"
    }
}

impl GetName for entity::Trip {
    fn get_name() -> &'static str {
        "Trip"
    }
}

impl GetName for entity::Complain {
    fn get_name() -> &'static str {
        "Complain"
    }
}

impl GetName for entity::Admin {
    fn get_name() -> &'static str {
        "Admin"
    }
}

impl DbConn {
    pub fn add<T>(&self, t: T) -> Result<Bson>
    where
        T: GetName + Serialize,
    {
        let coll = self.collection(T::get_name());
        to_doc(&t)
            .and_then(|doc| {
                coll.insert_one(doc, None).map_err(|err| {
                    ServiceError::MongodbError(err)
                })
            })
            .and_then(|r| {
                r.inserted_id.ok_or(ServiceError::String(
                    "ObjectId null".to_owned(),
                ))
            })
    }

    pub fn delete<T>(&self, id: &str) -> Result<()>
    where
        T: GetName + Serialize,
    {
        let coll = self.collection(T::get_name());
        let mut doc = Document::new();
        doc.insert("_id", id);
        coll.delete_one(doc, None)
            .map_err(|err| ServiceError::MongodbError(err))
            .map(|_| ())
    }
}

fn to_doc<T>(t: &T) -> Result<Document>
where
    T: Serialize,
{
    bson::to_bson(t)
        .map_err(|err| ServiceError::BsonEncoderError(err))
        .and_then(|doc| {
            doc.as_document().cloned().ok_or(ServiceError::String(
                "not document".to_string(),
            ))
        })
}