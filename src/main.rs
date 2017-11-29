#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate pin_che;
extern crate mongodb;

extern crate bson;
extern crate redis;

use mongodb::db::ThreadedDatabase;
use bson::Bson;
use redis::Commands;
use pin_che::{entity, db};
use pin_che::service::{Result, Service, ServiceError};
//use rocket::request::LenientForm;
use rocket_contrib::{Json, Value};

#[get("/")]
fn index(conn: db::DbConn, cache: db::CacheConn) -> String {
    let coll = conn.collection("admin");
    let name: String = cache.get("name").unwrap();
    println!("redis get name is {}", &name);

    let admin = entity::Admin {
        id: Some("sdfsf".to_owned()),
        name: String::from("lp"),
        pwd: String::from("123456"),
    };

    let serialized_person = bson::to_bson(&admin).unwrap(); // Serialize

    if let Bson::Document(document) = serialized_person {
        coll.insert_one(document, None).unwrap(); // Insert into a MongoDB collection
    } else {
        println!("Error converting the BSON object into a MongoDB document");
    }
    name
}

fn main() {
    rocket::ignite()
        .mount(
            "/",
            routes![index, register_owner, publish_trip, test_error],
        )
        .manage(pin_che::db::init_db_conn())
        .manage(pin_che::db::init_redis())
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
fn publish_trip(
    jwt: entity::JwtUser,
    form: entity::TripForm,
    s: Service,
) -> Result<Json<entity::Trip>> {
    let tel = s.get_tel(&jwt.id)?;
    let trip = entity::Trip::new(jwt.id, tel, form);
    s.publish_trip(&trip)?;
    Ok(Json(trip))
}

#[get("/test")]
fn test_error() -> Result<()> {
    Err(ServiceError::String(format!("{:?}",ServiceError::NoAuth)))
}