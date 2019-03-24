extern crate actix_web;
extern crate env_logger;
extern crate config;

use actix_web::{server, App, middleware};
use actix_web::http::Method;
use std::env;
use weather_service::web::AppState;
use std::sync::Arc;
use std::collections::HashMap;
use std::sync::Mutex;
use config::*;

fn main() {
    env::set_var("RUST_LOG", "actix_web=debug");
    env::set_var("RUST_LOG", "weather_service=info");

    env_logger::init();

    let mut settings = Config::default();
    settings.merge(File::with_name("cfg/config.json")).unwrap();

    let conf = Arc::new(Mutex::new(settings.try_into::<HashMap<String, String>>().unwrap()));

    server::new(move||
        App::with_state(AppState { config: conf.clone() })
            .middleware(middleware::Logger::default())
            .resource("/weather/{location}/on/{date}", |r| {
                r.method(Method::GET).a(weather_service::web::action::on_date::handle);
            })
            .resource("/weather/{location}/week", |r| {
                r.method(Method::GET).a(weather_service::web::action::on_week::handle);
            })
    )
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}
