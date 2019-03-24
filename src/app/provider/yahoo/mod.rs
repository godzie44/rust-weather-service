use futures::Future;
use actix_web::client;
use futures::future::err as fut_err;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use std::time::SystemTime;
use std::collections::BTreeMap;
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha1::Sha1;
use url::percent_encoding::{utf8_percent_encode, USERINFO_ENCODE_SET};
use chrono_tz::Tz;
use actix_web::client::ClientRequest;
use actix_web::Error;
use std::time::Duration;
use chrono::*;

use crate::app::{RawForecast};
use crate::app::provider::{WeatherProvider, ProviderError, WeatherProviderResponse};
use crate::app::forecast::Forecast;

use super::utils;

mod test;

define_encode_set! {
    pub FULL_ENCODE_SET = [USERINFO_ENCODE_SET] | {
        '&'
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct YahooResponse {
    forecasts: Vec<YahooDay>,
    location: YahooLocation,
}

impl WeatherProviderResponse for YahooResponse {
    fn to_forecast(&self) -> Forecast {
        let tz: Tz = self.location.timezone_id.parse().unwrap();

        Forecast::new(
            self.forecasts
                .iter()
                .map(|fd| {
                    let naive = NaiveDateTime::from_timestamp(fd.date, 0);
                    let utc_datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);

                    (
                        utc_datetime.with_timezone(&tz).format("%Y-%m-%d").to_string(),
                        fd.low + ((fd.high - fd.low) / 2.0),
                    )
                })
                .collect::<RawForecast>(),
            String::from(YahooProvider::NAME)
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct YahooLocation {
    timezone_id: String
}

#[derive(Debug, Serialize, Deserialize)]
struct YahooDay {
    date: i64,
    low: f64,
    high: f64,
}

impl WeatherProvider for YahooProvider {
    fn get_forecast(&self, location: &str) -> Box<super::ProviderForecastOption> {
        let yahoo_request = match self.build_request(location) {
            Ok(req) => req,
            Err(_) => return Box::new(fut_err(ProviderError::RequestError {
                reason: String::from("Unknown error!"),
                provider_name: Self::NAME.to_owned(),
            })),
        };

        Box::new(
            utils::fetch_forecast_request::<YahooResponse>(yahoo_request)
                .map(|res| {
                    info!("Forecast from Yahoo {:?}", res);
                    res
                })
                .map_err(|e| ProviderError::RequestError {
                    reason: e.to_string(),
                    provider_name: Self::NAME.to_owned(),
                })
                .then(|forecast_resp| Ok(forecast_resp))
        )
    }
}

pub struct YahooProvider {
    app_id: String,
    secret: String,
    user_key: String
}

impl YahooProvider {
    const BASE_URI: &'static str = "https://weather-ydn-yql.media.yahoo.com/forecastrss";
    const NAME: &'static str = "Yahoo";

    pub fn new(app_id: String, secret: String, user_key: String) -> Self {
        YahooProvider {
            app_id,
            secret,
            user_key
        }
    }

    fn build_request(&self, location: &str) -> Result<ClientRequest, Error> {
        client::get(self.build_forecast_url(location))
            .header("X-Yahoo-App-Id", self.app_id.clone())
            .header("Authorization", self.build_authorization_token(location))
            .finish()
    }

    fn build_forecast_url(&self, location: &str) -> String {
        format!("{}?location={}&format=json&u=c", Self::BASE_URI, location)
    }

    pub fn build_authorization_token(&self, location: &str) -> String {
        let mut parameters = Self::generate_oauth_parameters(&self.user_key);
        let base_string = Self::build_base_string(&parameters, location);

        let composite_key = format!("{}&", utf8_percent_encode(&self.secret, FULL_ENCODE_SET).to_string());

        let mut mac = Hmac::new(Sha1::new(), composite_key.as_bytes());
        mac.input(base_string.as_bytes());

        let oauth_signature = base64::encode(mac.result().code());

        parameters.insert("oauth_signature".to_owned(), oauth_signature.to_owned());

        let mut params_str: String = parameters.iter().map(|(key, value)| format!("{}={}, ", key, utf8_percent_encode(value, FULL_ENCODE_SET).to_string())).collect();
        let len = params_str.len();
        params_str.truncate(len - 2);

        format!("OAuth {}", params_str)
    }

    fn generate_oauth_parameters(user_key: &str) -> BTreeMap<String, String> {
        let mut hm: BTreeMap<String, String> = BTreeMap::new();
        hm.insert("oauth_consumer_key".to_owned(), user_key.to_owned());
        hm.insert("oauth_nonce".to_owned(), thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .collect::<String>());
        hm.insert("oauth_signature_method".to_owned(), "HMAC-SHA1".to_owned());
        hm.insert("oauth_timestamp".to_owned(), format!("{}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or(Duration::from_secs(0)).as_secs()));
        hm.insert("oauth_version".to_owned(), "1.0".to_owned());

        hm
    }

    fn build_base_string(params: &BTreeMap<String, String>, location: &str) -> String {
        let params_str: String = params.iter().map(|(key, value)| format!("&{}={}", key, value)).collect();

        let full_params = format!("format=json&location={}{}&u=c", location, params_str);

        format!(
            "GET&{}&{}",
            utf8_percent_encode(Self::BASE_URI, FULL_ENCODE_SET).to_string(),
            utf8_percent_encode(&full_params, FULL_ENCODE_SET).to_string(),
        )
    }
}
