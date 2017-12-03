#![feature(plugin, custom_derive,try_trait)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate lazy_static;
extern crate config;
extern crate mongodb;
extern crate serde;
extern crate rocket;
#[macro_use]
extern crate serde_derive;
extern crate redis;
extern crate r2d2;
extern crate r2d2_redis;
extern crate rustc_serialize;
extern crate jwt;
extern crate crypto;
extern crate bson;
extern crate serde_redis;
extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate hyper_tls;


pub mod setting;
pub mod db;
pub mod entity;
pub mod service;
pub mod external;