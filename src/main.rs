#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate diesel;
#[macro_use] extern crate rocket_contrib;
//use diesel::prelude::*;

pub mod schema;
pub mod models;

#[database("sqlite_logs")]
struct LogsDbConn(rocket_contrib::databases::diesel::SqliteConnection);

#[rocket::get("/world")]
fn world() -> &'static str {
    "Hello, world!"
}

fn main() {
    rocket::ignite()
        .attach(LogsDbConn::fairing())
        .mount("/hello", rocket::routes![world])
        .launch();
}
