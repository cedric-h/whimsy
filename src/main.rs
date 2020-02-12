#![feature(proc_macro_hygiene, decl_macro)]

#[rocket::get("/world")]
fn world() -> &'static str {
    "Hello, world!"
}

fn main() {
    rocket::ignite().mount("/hello", rocket::routes![world]).launch();
}
