#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate pin_che;
extern crate mongodb;

extern crate bson;
extern crate redis;

use mongodb::db::ThreadedDatabase;
use bson::Bson;
use bson::oid::ObjectId;
use redis::Commands;
use pin_che::{entity, db, service};
use pin_che::service::{Result, Service, ServiceError};
use rocket::request::LenientForm;
use rocket::response::content;

const OK: content::Json<&'static str> = content::Json("{'ok': true}");

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
        .mount("/", routes![index, register_owner, publish_trip])
        .manage(pin_che::db::init_db_conn())
        .manage(pin_che::db::init_redis())
        .launch();
}



#[get("/registerOwner?<user>")]
fn register_owner(
    jwt: entity::JwtUser,
    user: entity::OwnerForm,
    s: Service,
) -> Result<content::Json<&'static str>> {
    s.add_user(entity::User::new_owner(jwt.id, jwt.name, user))
        .map(|_| OK)
}

#[get("/publishTrip?<trip>")]
fn publish_trip(
    jwt: entity::JwtUser,
    trip: entity::TripForm,
    s: Service,
) -> Result<content::Json<&'static str>> {
    let tel = s.get_tel(&jwt.id)?;
    if jwt.role == "Owner" {
        s.publish_trip(entity::Trip::new(jwt.id, tel, trip)).map(
            |_| OK,
        )
    } else {
        Err(ServiceError::NoAuth)
    }
}