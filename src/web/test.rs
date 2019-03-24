#![cfg(test)]
extern crate config;

use actix_web::{http, test, HttpMessage};
use crate::web::action::*;
use actix_web::App;
use crate::web::AppState;
use config::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;
use crate::web;
use chrono::Utc;
use time::Duration;

fn create_app() -> App<web::AppState> {
    let mut settings = Config::default();
    settings.merge(File::with_name("cfg/config_test.json")).unwrap();
    let conf = Arc::new(Mutex::new(settings.try_into::<HashMap<String, String>>().unwrap()));

    App::with_state(AppState { config: conf.clone() })
        .resource("/test_week/{location}", |r| r.h(on_week::handle))
        .resource("/test_date/{location}/{date}", |r| r.h(on_date::handle))
}

#[test]
fn test_on_week_action() {
    let mut srv = test::TestServer::with_factory(create_app);

    let request = srv.client(http::Method::GET, "/test_week/Moscow").finish().unwrap();
    let response = srv.execute(request.send()).unwrap();

    let body_bytes = srv.execute(response.body()).unwrap();

    assert!(response.status().is_success());
    assert_body_ok(::std::str::from_utf8(&body_bytes).unwrap());
}

#[test]
fn test_on_date_action() {
    let day = (Utc::now() + Duration::days(1)).format("%Y-%m-%d").to_string();

    let mut srv = test::TestServer::with_factory(create_app);

    let request = srv.client(http::Method::GET, &format!("/test_date/Moscow/{}", day)).finish().unwrap();
    let response = srv.execute(request.send()).unwrap();

    let body_bytes = srv.execute(response.body()).unwrap();

    assert!(response.status().is_success());
    assert_body_ok(::std::str::from_utf8(&body_bytes).unwrap());
}

#[test]
fn test_on_week_action_fail() {
    let mut srv = test::TestServer::with_factory(create_app);

    let request = srv.client(http::Method::GET, "/test_week/UnknownCityInUnknownCountry").finish().unwrap();
    let response = srv.execute(request.send()).unwrap();

    let body_bytes = srv.execute(response.body()).unwrap();

    assert!(response.status().is_success());
    assert_body_err(::std::str::from_utf8(&body_bytes).unwrap());
}

#[test]
fn test_on_week_ok_but_with_warning() {
    let mut srv = test::TestServer::with_factory(create_app);

    //yahoo know about ascx city, apixu don't
    let request = srv.client(http::Method::GET, "/test_week/ascx").finish().unwrap();
    let response = srv.execute(request.send()).unwrap();

    let body_bytes = srv.execute(response.body()).unwrap();

    assert!(response.status().is_success());
    assert_body_ok(::std::str::from_utf8(&body_bytes).unwrap());
    assert_body_warning_count(::std::str::from_utf8(&body_bytes).unwrap(), 1);
}

fn assert_body_ok(as_string: &str) {
    let json: ForecastUserResponse = serde_json::from_str::<ForecastUserResponse>(as_string).unwrap();

    assert!(json.ok);
}

fn assert_body_err(as_string: &str) {
    let json: ForecastUserResponse = serde_json::from_str::<ForecastUserResponse>(as_string).unwrap();

    assert_eq!(false, json.ok);
}

fn assert_body_warning_count(as_string: &str, count: i32) {
    let json: ForecastUserResponse = serde_json::from_str::<ForecastUserResponse>(as_string).unwrap();

    assert_eq!(count, json.warnings.len() as i32);
}