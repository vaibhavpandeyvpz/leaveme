mod routes;
mod slack;

#[macro_use]
extern crate rocket;

use clap::{arg, command, ArgMatches};
use config::Config;
use std::any::Any;

lazy_static::lazy_static! {
    static ref CLIARGS: ArgMatches = command!()
        .arg(
            arg!(
                -c --config <FILE> "Path to config file"
            )
            .required(true)
        )
        .get_matches();

    static ref CONFIG: Config = Config::builder()
        .add_source(config::File::new(cliarg::<String>("config").as_str(), config::FileFormat::Yaml))
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .unwrap();
}

pub(crate) fn cliarg<T: Any + Clone + Send + Sync + 'static>(key: &str) -> T {
    CLIARGS.get_one::<T>(key).unwrap().clone()
}

pub(crate) fn config<'a, T: serde::Deserialize<'a>>(key: &str) -> T {
    CONFIG.get::<T>(key).unwrap()
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount(
        "/",
        routes![
            routes::index,
            routes::slack_command,
            routes::slack_interaction,
        ],
    )
}
