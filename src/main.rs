#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate pin_che;
extern crate mongodb;

extern crate bson;
extern crate redis;

use mongodb::ThreadedClient;
use mongodb::db::ThreadedDatabase;
use bson::Bson;
use redis::Commands;
use pin_che::entity;

#[get("/")]
fn index(conn: pin_che::db::DbConn, cache: pin_che::db::CacheConn) -> String {
    let coll = conn.db("test").collection("admin");
    let name: String = cache.get("name").unwrap();
    println!("redis get name is {}", &name);

    let admin = entity::Admin {
        id: None,
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
        .mount("/", routes![index])
        .manage(pin_che::db::init_db_conn())
        .manage(pin_che::db::init_redis())
        .launch();
}



#[get("/registerOwner")]
fn register_owner() -> String {
    String::from("s")
}

