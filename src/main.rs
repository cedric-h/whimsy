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

    #[rocket::put("/<title>", data = "<body>")]
    pub fn submit_whim(conn: DbConn, title: String, body: String) -> Result<(), String> {
        diesel::insert_into(whims::table)
            .values(&models::Whim { title, body })
            .execute(&*conn)
            .map_err(|e| format!("database err: {}", e))?;

        Ok(())
    }

    #[rocket::get("/<_title>")]
    pub fn whim_client(_title: String) -> NamedFile {
        NamedFile::open(std::path::Path::new("front/dist/index.html"))
            .expect("Couldn't open whimsy client index.html in 'front/dist/index.html'")
    }
}

fn main() {
    use rocket::routes;
    use rocket_contrib::serve::StaticFiles;

    rocket::ignite()
        .attach(whim::DbConn::fairing())
        .mount("/raw/whim", routes![whim::get_whim])
        .mount("/whim", routes![whim::submit_whim, whim::whim_client])
        .mount("/whim/pub", StaticFiles::from("front/dist"))
        .launch();
}
