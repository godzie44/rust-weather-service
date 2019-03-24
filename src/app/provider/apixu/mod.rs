use futures::Future;
use crate::app::provider::{WeatherProvider, ProviderError};
use actix_web::client;
use futures::future::err as fut_err;
use crate::app::provider::WeatherProviderResponse;
use super::utils;
use actix_web::client::ClientRequest;
use actix_web::Error;
use crate::app::RawForecast;
use crate::app::forecast::Forecast;

mod test;

#[derive(Debug, Serialize, Deserialize)]
struct ApixuResponse {
    forecast: ApixuForecast,
}

impl WeatherProviderResponse for ApixuResponse {
    fn to_forecast(&self) -> Forecast {
        Forecast::new(
            self.forecast.forecastday
                .iter()
                .map(|fd| (fd.date.clone(), fd.day.avgtemp_c))
                .collect::<RawForecast>(),
            String::from(ApixuProvider::NAME)
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ApixuForecast {
    forecastday: Vec<ApixuForecastDay>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApixuForecastDay {
    date: String,
    day: ApixuDay,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApixuDay {
    avgtemp_c: f64,
    avgtemp_f: f64,
}

pub struct ApixuProvider {
    key: String
}

impl ApixuProvider {
    const BASE_URI: &'static str = "http://api.apixu.com/v1/forecast.json";
    const NAME: &'static str = "Apixu";

    pub fn new(key: String) -> Self {
        ApixuProvider {key}
    }

    fn build_request(&self, location: &str) -> Result<ClientRequest, Error> {
        client::get(format!("{}?key={}&q={}&days=7", Self::BASE_URI, self.key, location)).finish()
    }
}

impl WeatherProvider for ApixuProvider {
    fn get_forecast(&self, location: &str) -> Box<super::ProviderForecastOption> {
        let apixu_request = match self.build_request(location) {
            Ok(req) => req,
            Err(_) => return Box::new(fut_err(ProviderError::RequestError {
                reason: String::from("Inner error!"),
                provider_name: Self::NAME.to_owned(),
            })),
        };

        Box::new(
            utils::fetch_forecast_request::<ApixuResponse>(apixu_request)
                .map(|res| {
                    info!("Forecast from Apixu {:?}", res);
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