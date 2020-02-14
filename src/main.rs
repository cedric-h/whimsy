#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate diesel;
#[macro_use] extern crate rocket_contrib;

pub mod schema;
pub mod models;

use schema::whims;
use whims::dsl::whims as all_whims;
use diesel::prelude::*;


#[database("whims")]
struct WhimsDbConn(diesel::SqliteConnection);

#[rocket::get("/<id>")]
fn get_whim(db: WhimsDbConn, id: usize) -> String {
    all_whims
        .order(whims::id.desc())
        .load::<models::Whim>(&*db)
        .expect("no whims fetch")
        .get(id)
        .expect("no whims")
        .body
        .clone()
}

fn main() {
    rocket::ignite()
        .attach(WhimsDbConn::fairing())
        .mount("/whims", rocket::routes![get_whim])
        .launch();
}
