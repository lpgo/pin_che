#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate pin_che;
extern crate mongodb;

extern crate bson;
extern crate redis;
extern crate tokio_timer;
extern crate futures;

use pin_che::{entity, db};
use pin_che::service::{Result, Service, ServiceError};
//use rocket::request::LenientForm;
use rocket_contrib::{Json, Value};
use std::time::Duration;
use tokio_timer::Timer;
use futures::{Stream, Future};
use std::thread;

fn main() {
    let database = pin_che::db::init_db_conn();
    let pool = pin_che::db::init_redis();
    let service = Service::new(
        db::DbConn(database.clone()),
        db::CacheConn(pool.get().unwrap()),
    );

    thread::spawn(|| {
        let timer = Timer::default();
        let interval = timer.interval(Duration::from_millis(1000));
        interval
            .for_each(move |_| {
                service.test();
                Ok(())
            })
            .wait()
            .unwrap();
    });

    thread::spawn(|| {
        println!("{:?}", pin_che::db::check_expire());
    });

    rocket::ignite()
        .mount(
            "/",
            routes![
                publish_trip,
                test_request,
                apply_trip,
                pay,
                discount,
                submit,
                get_trips,
            ],
        )
        .manage(database)
        .manage(pool)
        .catch(errors![not_found, noauth])
        .launch();
}

#[error(404)]
fn not_found() -> Json<Value> {
    Json(json!({
        "status": "error",   //ok表示逻辑错误，系统运行正常；error表示系统内部运行错误（可能是参数解析错误）
        "reason": "Resource was not found."
    }))
}

#[error(401)]
fn noauth() -> Result<()> {
    Err(ServiceError::NoAuth)
}

#[get("/publishTrip?<form>")]
fn publish_trip(form: entity::TripForm, s: Service) -> Result<Json<entity::Trip>> {
    //let tel = s.get_tel(&jwt.id)?;
    let trip = entity::Trip::new("openid".to_owned(), form);
    s.publish_trip(&trip)?;
    Ok(Json(trip))
}

#[get("/applyTrip/<id>/<count>/<tel>")]
fn apply_trip(
    id: String,
    count: i64,
    tel: Option<String>,
    s: Service,
) -> Result<Json<entity::Order>> {
    s.apply_trip(id, "openid".to_owned(), count, tel).map(
        |order| {
            Json(order)
        },
    )
}

//微信支付回调（通过其他服务）
#[get("/pay/<id>/<sign>")]
fn pay(id: String, sign: String, s: Service) -> Result<()> {
    s.pay(id, sign)
}

#[get("/discount/<id>/<fee>")]
fn discount(id: String, fee: i64, s: Service) -> Result<()> {
    s.discount(id, fee)
}

#[get("/submit/<id>")]
fn submit(id: String, s: Service) -> Result<()> {
    s.submit(id)
}
#[get("/getTrips/<page>")]
fn get_trips(s: Service, page: isize) -> Result<Json<Vec<entity::Trip>>> {
    s.get_trips(page).map(|vec| Json(vec))
}

#[get("/test/request")]
fn test_request() -> Result<()> {
    pin_che::external::test()
}