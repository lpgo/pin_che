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
        println!("start timer");
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

    rocket::ignite()
        .mount(
            "/",
            routes![
                register_owner,
                publish_trip,
                test_request,
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
        "status": "error",
        "reason": "Resource was not found."
    }))
}

#[error(401)]
fn noauth() -> Result<()> {
    Err(ServiceError::NoAuth)
}

#[get("/registerOwner?<user>")]
fn register_owner(
    jwt: entity::JwtUser,
    user: entity::OwnerForm,
    s: Service,
) -> Result<Json<entity::User>> {
    let owner = entity::User::new_owner(jwt.id, jwt.name, user);
    s.add_user(&owner).map(|_| Json(owner))
}

#[get("/publishTrip?<form>")]
fn publish_trip(form: entity::TripForm, s: Service) -> Result<Json<entity::Trip>> {
    //let tel = s.get_tel(&jwt.id)?;
    let trip = entity::Trip::new("openid".to_owned(), "tel".to_owned(), form);
    s.publish_trip(&trip)?;
    Ok(Json(trip))
}

#[get("/test/request")]
fn test_request() -> Result<()> {
    pin_che::external::test()
}