#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate pin_che;
extern crate mongodb;
#[macro_use(bson, doc)]
extern crate bson;
extern crate redis;

use mongodb::ThreadedClient;
use mongodb::db::ThreadedDatabase;
use bson::Bson;
use redis::Commands;

#[get("/")]
fn index(conn: pin_che::db::DbConn, cache: pin_che::db::CacheConn) -> String {
    let coll = conn.db("test").collection("movies");

    let doc =
        doc! { "title" => "Jaws",
                      "array" => [ 1, 2, 3 ] };

    // Insert document into 'test.movies' collection
    coll.insert_one(doc.clone(), None).ok().expect(
        "Failed to insert document.",
    );

    // Find the document and receive a cursor
    let mut cursor = coll.find(Some(doc.clone()), None).ok().expect(
        "Failed to execute find.",
    );

    let item = cursor.next();

    let name: String = cache.get("name").unwrap();
    println!("{}", name);

    // cursor.next() returns an Option<Result<Document>>
    match item {
        Some(Ok(doc)) => {
            match doc.get("title") {
                Some(&Bson::String(ref title)) => return title.clone(),
                _ => panic!("Expected title to be a string!"),
            }
        }
        Some(Err(_)) => panic!("Failed to get next from server!"),
        None => panic!("Server returned no results!"),
    };
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index])
        .manage(pin_che::db::init_db_conn())
        .manage(pin_che::db::init_redis())
        .launch();
}