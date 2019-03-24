use actix_web::client::ClientRequest;
use futures::Future;
use actix_web::HttpMessage;
use crate::app::provider::WeatherProviderResponse;
use serde::de::DeserializeOwned;
use std::time::Duration;
use crate::app::Forecast;

#[derive(Debug, Fail)]
pub enum RequestError {
    #[fail(display = "Request Unknown error")]
    UnknownError {},

    #[fail(display = "Request time out")]
    ProviderTimeOut {},

    #[fail(display = "Invalid response (invalid location)")]
    InvalidResponse {},
}

pub fn fetch_forecast_request<T: 'static>(request: ClientRequest) -> Box<Future<Item=Forecast, Error=RequestError>>
    where T: WeatherProviderResponse + DeserializeOwned
{
    Box::new(
        request
            .send()
            .timeout(Duration::new(10, 0))
            .map_err(|_| RequestError::ProviderTimeOut {})
            .and_then(|response| {
                response
                    .body()
                    .map_err(|_| RequestError::UnknownError {})
                    .and_then(|body| {
                        serde_json::from_slice::<T>(&body).map_err(|_| RequestError::InvalidResponse {}).map(|resp| resp.to_forecast())
                    })
            })
    )
}