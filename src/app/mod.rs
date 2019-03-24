use self::provider::WeatherProvider;
use futures::Future;
use crate::app::provider::ProviderError;
use std::collections::BTreeMap;
use futures::future::join_all;
use crate::app::forecast::ForecastAggregate;
use crate::app::forecast::Forecast;

pub mod provider;
pub mod forecast;

type Date = String;
type Temperature = f64;

type RawForecast = BTreeMap<Date, Temperature>;

type ForecastAggregateResponse = Future<Item=ForecastAggregate, Error=ProviderError>;

pub struct WeatherAggregateManager {
    providers: Vec<Box<WeatherProvider>>
}

impl WeatherAggregateManager {
    pub fn new(providers: Vec<Box<WeatherProvider>>) -> Self {
        WeatherAggregateManager {
            providers
        }
    }

    pub fn get_forecast_aggregate_at(&self, date: Date, location: &str) -> Box<ForecastAggregateResponse> {
        Box::new(
            self
                .get_forecast_aggregate(location)
                .map(move |aggregate| {
                    aggregate.filter_by_date(date)
                })
        )
    }

    pub fn get_forecast_aggregate_on_week(&self, location: &str) -> Box<ForecastAggregateResponse> {
        Box::new(
            self
                .get_forecast_aggregate(location)
                .map(|aggregate| {
                    aggregate.into_week_aggregate()
                })
        )
    }

    fn get_forecast_aggregate(&self, location: &str) -> Box<ForecastAggregateResponse> {
        Box::new(
            join_all(self.get_forecast_future_list(location))
                .map(|forecast_list: Vec<Result<Forecast, ProviderError>>|
                    forecast_list.into_iter().fold(ForecastAggregate::empty(), |aggregate, forecast_opt| {
                        aggregate.with_forecast_result(forecast_opt)
                    })
                )
        )
    }

    fn get_forecast_future_list(&self, location: &str) -> Vec<Box<self::provider::ProviderForecastOption>> {
        self
            .providers
            .iter()
            .map(|provider|
                provider.get_forecast(location)
            )
            .collect::<Vec<Box<self::provider::ProviderForecastOption>>>()
    }
}


#[cfg(test)]
mod manager_test {
    use crate::app::forecast::Forecast;
    use crate::app::RawForecast;
    use crate::app::provider::WeatherProvider;
    use futures::Future;
    use crate::app::provider::ProviderError;
    use super::*;

    fn create_forecast() -> Forecast {
        Forecast::new(
            vec![(String::from("2019-03-01"), 3.0), (String::from("2019-03-02"), 4.0), (String::from("2019-03-03"), 5.0), (String::from("2019-03-04"), 6.0), (String::from("2019-03-05"), 7.0), (String::from("2019-03-06"), 8.0), (String::from("2019-03-07"), 9.0)].into_iter().collect::<RawForecast>(),
            String::from("provider_stub"),
        )
    }

    struct ProviderStub;

    impl WeatherProvider for ProviderStub {
        fn get_forecast(&self, _location: &str) -> Box<Future<Item=Result<Forecast, ProviderError>, Error=ProviderError>> {
            Box::new(
                futures::future::ok(
                    Ok(self::create_forecast())
                )
            )
        }
    }

    #[test]
    fn test_get_at_date_work_as_expected() {
        let wam = WeatherAggregateManager::new(vec![
            Box::new(ProviderStub {}),
            Box::new(ProviderStub {})
        ]);

        let result = wam
            .get_forecast_aggregate_at(String::from("2019-03-01"), "location")
            .wait();


        assert_eq!(
            result.unwrap().calculate_average_forecast().unwrap().get_temperature_at(String::from("2019-03-01")).unwrap(),
            &3.0
        );
    }

    #[test]
    fn test_get_on_week_work_as_expected() {
        let wam = WeatherAggregateManager::new(vec![
            Box::new(ProviderStub {}),
            Box::new(ProviderStub {})
        ]);

        let result = wam.get_forecast_aggregate_on_week("location").wait();

        let forecast = result.unwrap().calculate_average_forecast().unwrap();
        assert_eq!(
            forecast.get_temperature_at(String::from("2019-03-01")).unwrap(),
            &3.0
        );
        assert_eq!(
            forecast.get_temperature_at(String::from("2019-03-05")).unwrap(),
            &7.0
        );
    }

}