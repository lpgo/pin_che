use mongodb::{Client, ThreadedClient};
use mongodb::db::{Database, ThreadedDatabase};
use std::ops::Deref;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use rocket::http::Status;
use r2d2;
use r2d2_redis::RedisConnectionManager;
use redis::{self, Connection, PipelineCommands, Commands};
use setting;
use service::{ServiceError, Result};
use entity;
use serde::ser::Serialize;
use serde::de::Deserialize;
use serde_redis::RedisDeserialize;
use bson::{self, Document, Bson};


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

impl DbConn {
    pub fn add<T>(&self, t: &T) -> Result<Bson>
    where
        T: GetName + Serialize,
    {
        let coll = self.collection(T::get_name());
        to_doc(t)
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

    pub fn get_one<'de, T>(&self, id: &str) -> Result<T>
    where
        T: GetName + Deserialize<'de>,
    {
        let coll = self.collection(T::get_name());
        let mut doc = Document::new();
        doc.insert("_id", id);
        let doc = coll.find_one(Some(doc), None)??;
        bson::from_bson::<T>(Bson::Document(doc)).map_err(|err| ServiceError::BsonDecoderError(err))
    }
}

impl CacheConn {
    pub fn add_trip(&self, t: &entity::Trip) -> Result<()> {
        println!("{:?}", t);
        let mut pipe = redis::pipe();
        pipe.atomic()
            .hset_multiple(
                format!("{}:{}", entity::Trip::get_name(), t.id),
                &[
                    ("_id", &t.id),
                    ("openid", &t.openid),
                    ("start", &t.start),
                    ("end", &t.end),
                    ("venue", &t.venue),
                    ("plate_number", &t.plate_number),
                    ("car_type", &t.car_type),
                    ("tel", &t.tel),
                ],
            )
            .hset_multiple(
                format!("{}:{}", entity::Trip::get_name(), t.id),
                &[
                    ("seat_count", t.seat_count),
                    ("current_seat", t.current_seat),
                    ("start_time", t.start_time),
                    ("price", t.price),
                ],
            )
            .hset(
                format!("{}:{}", entity::Trip::get_name(), t.id),
                "status",
                &t.status,
            );
        if let Some(ref msg) = t.message {
            pipe.hset(
                format!("{}:{}", entity::Trip::get_name(), t.id),
                "message",
                msg,
            );
        }
        pipe.query(&**self)
            .map(|result: Vec<i32>| {
                println!("redis add trip result is {:?}", result)
            })
            .map_err(|err| ServiceError::RedisError(err))
    }

    pub fn add_order(&self, order: &entity::Order) -> Result<()> {
        let trip_key = format!("{}:{}", entity::Trip::get_name(), order.trip_id);
        let order_key = format!("{}:{}", entity::Order::get_name(), order.id);
        redis::transaction(&**self, &[&trip_key], |pipe| {
            let count: i64 = self.hget(&trip_key, "current_seat")?;
            if count < order.count {
                return pipe.query(&**self).map(|_: Vec<i32>| Some(false));
            }
            pipe.hincr(&trip_key, "current_seat", -order.count)
                .hset_multiple(
                    &order_key,
                    &[
                        ("_id", &order.id),
                        ("openid", &order.openid),
                        ("trip_id", &order.trip_id),
                        ("trip_owner",&order.trip_owner),
                    ],
                )
                .hset_multiple(
                    &order_key,
                    &[
                        ("order_id", order.order_id.as_ref()),
                        ("transaction_id", order.transaction_id.as_ref()),
                        ("tel", order.tel.as_ref()),
                    ],
                )
                .hset_multiple(
                    &order_key,
                    &[
                        ("price", order.price),
                        ("count", order.count),
                        ("start_time", order.start_time),
                    ],
                )
                .hset(&order_key, "status", &order.status)
                .expire(&order_key, 3 * 60)
                .hset(format!("OrderEx:{}", order.id), "count",order.count)
                .hset(format!("OrderEx:{}", order.id),"trip_id",&order.trip_id)  //用于未支付时恢复物品数量
                .query(&**self)
                .map(|_: Vec<i32>| Some(true))
        }).map_err(|err| ServiceError::RedisError(err))
            .and_then(|buy| if buy {
                Ok(())
            } else {
                Err(ServiceError::DontHaveEnoughSeats)
            })
    }

    pub fn pay_order(&self, id: String) -> Result<()> {
        let order_key = format!("{}:{}", entity::Order::get_name(), &id);
        redis::pipe()
            .atomic()
            .hset(order_key, "status", &entity::OrderStatus::Paid)
            .del(format!("OrderEx:{}", &id))
            .query(&**self)
            .map(|_: Vec<i32>| ())
            .map_err(|err| ServiceError::RedisError(err))
    }

    //返回transaction_id用于微信退款
    pub fn change_order_price(&self, order_id: &str, openid: &str, change: i32) -> Result<String> {
        let order_key = format!("{}:{}", entity::Order::get_name(), order_id);
        let trip_owner: String = self.hget(&order_key, "trip_owner")?;
        let transaction_id: String = self.hget(&order_key, "transaction_id")?;
        if openid != trip_owner {
            Err(ServiceError::TripNotYours)
        } else {
            self.hincr(&order_key, "price", change)
                .map(|_: i32| transaction_id)
                .map_err(|err| ServiceError::RedisError(err))
        }
    }

    pub fn get_object<'de, T>(&self, id: &str) -> Result<T>
    where
        T: GetName + Deserialize<'de>,
    {
        let value: redis::Value = self.hgetall(format!("{}:{}", "Trip", id))?;
        value.deserialize().map_err(
            |err| ServiceError::RedisDecodeError(err),
        )
    }
}

pub fn check_expire() -> Result<()> {
    let client = redis::Client::open(setting::get_str("app.redis").as_str())?;
    let mut pubsub = client.get_pubsub()?;
    pubsub.subscribe("__keyevent@*__:expired")?;
    loop {
        let msg = pubsub.get_message()?;
        let key: String = msg.get_payload()?;
        let v: Vec<&str> = key.split(":").collect();
        let ex_key = format!("OrderEx:{}", v[1]);
        let trip_id: String = client.hget(&ex_key, "trip_id")?;
        let count: i32 = client.hget(&ex_key, "count")?;
        let _: i32 = client.hincr(
            format!("Trip:{}", trip_id),
            "current_seat",
            count,
        )?;
        let _: i32 = client.del(&ex_key)?;
    }
}