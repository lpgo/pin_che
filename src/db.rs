use mongodb::{Client, ThreadedClient};
use std::ops::Deref;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use rocket::http::Status;
use r2d2;
use r2d2_redis::RedisConnectionManager;
use redis::Connection;
use setting;


type Pool = r2d2::Pool<RedisConnectionManager>;

//mongodb client
pub struct DbConn(pub Client);
pub struct CacheConn(pub r2d2::PooledConnection<RedisConnectionManager>);

impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, ()> {
        let pool = request.guard::<State<Client>>()?;
        Outcome::Success(DbConn(pool.clone()))
    }
}

impl Deref for DbConn {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn init_db_conn() -> Client {
    Client::connect(
        &setting::get_str("app.dburl"),
        setting::get_int64("app.dbport") as u16,
    ).expect("can't connect db")
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