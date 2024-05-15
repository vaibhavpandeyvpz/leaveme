#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Leave me please."
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
