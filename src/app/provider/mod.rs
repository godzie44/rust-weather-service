use crate::app::Date;
use crate::app::forecast::Forecast;

pub mod apixu;
pub mod yahoo;
pub mod utils;

#[derive(Debug, Fail)]
pub enum ProviderError {
    #[fail(display = "Reason:{}, provider: {}!", reason, provider_name)]
    RequestError {
        reason: String,
        provider_name: String,
    },

    #[fail(display = "Unsupported date {} for provider {} !", date, provider_name)]
    UnsupportedDate {
        date: Date,
        provider_name: String,
    },
}

pub type ProviderForecastOption = futures::Future<Item=Result<Forecast, ProviderError>, Error=ProviderError>;

pub trait WeatherProviderResponse {
    fn to_forecast(&self) -> Forecast;
}

pub trait WeatherProvider {
    fn get_forecast(&self, location: &str) -> Box<ProviderForecastOption>;
}