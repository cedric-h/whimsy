#![feature(try_trait, proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate diesel;
#[macro_use] extern crate rocket_contrib;

pub mod schema;
pub mod models;

mod whim {
    use diesel::prelude::*;
    use crate::models;
    use crate::schema::whims;
    use whims::dsl::{whims as all_whims, title as whim_title};
    use rocket::response::NamedFile;

    #[database("whims")]
    pub struct DbConn(diesel::SqliteConnection);

    #[rocket::get("/<title>")]
    pub fn get_whim(conn: DbConn, title: String) -> Option<String> {
        all_whims
            .filter(whim_title.eq(&title))
            .load::<models::Whim>(&*conn)
            .expect("Couldn't lock DB to get whim")
            .pop()
            .map(|w| w.body)
    }

    #[rocket::get("/<_title>")]
    pub fn whim_client(_title: String) -> NamedFile {
        NamedFile::open(std::path::Path::new("front/dist/index.html"))
            .expect("Couldn't open whimsy client index.html in 'front/dist/index.html'")
    }
}

fn main() {
    rocket::ignite()
        .attach(whim::DbConn::fairing())
        .mount("/raw/whim", rocket::routes![whim::get_whim])
        .mount("/whim", rocket::routes![whim::whim_client])
        .mount("/whim/pub", rocket_contrib::serve::StaticFiles::from("front/dist"))
        .launch();
}
