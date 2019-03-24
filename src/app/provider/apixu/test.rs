#![cfg(test)]
extern crate config;
use config::*;

use super::*;
use actix::System;
use chrono::DateTime;
use chrono::Utc;
use time::Duration;
use std::collections::HashMap;

fn create_apixu_provider() -> ApixuProvider {
    let mut settings = Config::default();
    settings.merge(File::with_name("cfg/config_test.json")).unwrap();
    let conf = settings.try_into::<HashMap<String, String>>().unwrap();

    ApixuProvider::new(
        conf.get("apixu_key").unwrap().clone()
    )
}

#[test]
fn test_apixu_return_forecast() {
    let result_fut = create_apixu_provider().get_forecast("Moscow");

    let mut ctx = System::new("test");
    let response = ctx.block_on(result_fut);
    assert!(response.is_ok());

    let forecast_opt = response.unwrap();
    assert!(forecast_opt.is_ok());

    let dt: DateTime<Utc> = Utc::now() + Duration::days(1);
    let day = dt.format("%Y-%m-%d").to_string();

    assert!(forecast_opt.unwrap().get_temperature_at(day).is_some());
}

#[test]
fn test_apixu_error_for_invalid_location() {
    let result_fut = create_apixu_provider().get_forecast("UnknownCityInUnknownCountry");

    let mut ctx = System::new("test");
    let response = ctx.block_on(result_fut);
    assert!(response.is_ok());

    let forecast_opt = response.unwrap();
    assert!(forecast_opt.is_err());
}