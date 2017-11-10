#![feature(plugin)]
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

pub mod setting;
pub mod db;
pub mod entity;