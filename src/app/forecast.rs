use std::collections::BTreeMap;
use crate::app::Date;
use crate::app::Temperature;
use crate::app::RawForecast;

#[derive(Debug, Fail)]
pub enum ForecastError {
    #[fail(display = "Unsupported date {} for provider {} !", date, provider_name)]
    UnsupportedDate {
        date: Date,
        provider_name: String,
    },

    #[fail(display = "To short forecast for provider {} !", provider_name)]
    ToShortForecast {
        provider_name: String,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Forecast {
    #[serde(flatten)]
    items: BTreeMap<Date, Temperature>,

    #[serde(skip)]
    source_name: String,
}

impl Forecast {
    pub fn new(raw_forecast: RawForecast, source_name: String) -> Self {
        Forecast { items: raw_forecast, source_name }
    }

    pub fn get_temperature_at(&self, date: Date) -> Option<&Temperature> {
        self.items.get(&date)
    }

    pub fn into_date_forecast(self, date: Date) -> Result<Forecast, ForecastError> {
        match self.items.get(&date) {
            Some(temp) => Ok(Self::new(vec![(date, *temp)].into_iter().collect(), self.source_name)),
            None => Err(ForecastError::UnsupportedDate {
                date: date.clone(),
                provider_name: self.source_name,
            })
        }
    }

    pub fn into_week_forecast(self) -> Result<Forecast, ForecastError> {
        let raw: RawForecast = self.items.into_iter().take(5).collect();

        match raw.len() == 5 {
            true => Ok(Self::new(raw, self.source_name)),
            _ => Err(ForecastError::ToShortForecast { provider_name: self.source_name })
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ForecastAggregate {
    forecast_collection: Vec<Forecast>,
    warnings: Vec<String>,
}

impl ForecastAggregate {
    pub fn new(forecast_collection: Vec<Forecast>, warnings: Vec<String>) -> Self {
        ForecastAggregate {
            forecast_collection,
            warnings,
        }
    }

    pub fn empty() -> Self {
        ForecastAggregate::new(Vec::new(), Vec::new())
    }

    pub fn get_warnings(&self) -> &Vec<String> {
        &self.warnings
    }

    pub fn with_forecast_collection(self, forecast_c: Vec<Forecast>) -> Self {
        ForecastAggregate::new(
            vec![self.forecast_collection, forecast_c].into_iter().flat_map(|s| s.into_iter()).collect(),
            self.warnings,
        )
    }

    pub fn with_warning_collection(self, warning_c: Vec<String>) -> Self {
        ForecastAggregate::new(
            self.forecast_collection,
            vec![self.warnings, warning_c].into_iter().flat_map(|s| s.into_iter()).collect(),
        )
    }

    pub fn with_forecast_result<E: failure::Fail>(self, forecast_opt: Result<Forecast, E>) -> Self {
        match forecast_opt {
            Ok(forecast) => self.with_forecast_collection(vec![forecast]),
            Err(e) => self.with_warning_collection(vec![e.to_string()])
        }
    }

    pub fn filter_by_date(self, date: Date) -> Self {
        self.forecast_collection.into_iter().fold(ForecastAggregate::empty(), move |aggregate, forecast| {
            aggregate.with_forecast_result(forecast.into_date_forecast(date.clone()))
        })
            .with_warning_collection(self.warnings)
    }


    pub fn into_week_aggregate(self) -> Self {
        self.forecast_collection.into_iter().fold(ForecastAggregate::empty(), |aggregate, forecast| {
            aggregate.with_forecast_result(forecast.into_week_forecast())
        })
            .with_warning_collection(self.warnings)
    }

    pub fn calculate_average_forecast(&self) -> Option<Forecast> {
        if self.forecast_collection.len() == 0 {
            return None;
        }

        let mut avg_forecast = Forecast::new(RawForecast::new(), String::from("multiple"));
        self.forecast_collection
            .iter()
            .for_each(|current_forecast| {
                current_forecast.items.iter().for_each(|(date, temp)| {
                    avg_forecast
                        .items
                        .entry(date.to_owned())
                        .and_modify(|t| { *t += temp })
                        .or_insert(*temp);
                });
            });

        avg_forecast
            .items
            .iter_mut()
            .for_each(|(_date, temperature)| {
                *temperature = *temperature / (self.forecast_collection.len() as f64).round();
            });

        Some(avg_forecast)
    }
}

#[cfg(test)]
mod forecast_aggregate_test {
    use crate::app::forecast::Forecast;
    use crate::app::RawForecast;
    use crate::app::forecast::ForecastAggregate;
    use crate::app::Date;
    use crate::app::Temperature;

    fn create_forecast() -> Forecast {
        Forecast::new(
            vec![(String::from("2019-03-01"), 3.0), (String::from("2019-03-02"), 4.0), (String::from("2019-03-03"), 5.0), (String::from("2019-03-04"), 6.0), (String::from("2019-03-05"), 7.0), (String::from("2019-03-06"), 8.0), (String::from("2019-03-07"), 9.0)].into_iter().collect::<RawForecast>(),
            String::from("test"),
        )
    }

    fn create_forecast_on_date(date: Date, temp: Temperature) -> Forecast {
        Forecast::new(
            vec![(date, temp)].into_iter().collect::<RawForecast>(),
            String::from("test"),
        )
    }

    #[test]
    fn test_forecast_aggregate_filter_by_date_work_as_expected() {
        let aggregate = ForecastAggregate::new(vec![self::create_forecast(), self::create_forecast()], vec![])
            .filter_by_date(String::from("2019-03-02"));

        assert_eq!(aggregate.forecast_collection, vec![
            self::create_forecast_on_date(String::from("2019-03-02"), 4.0),
            self::create_forecast_on_date(String::from("2019-03-02"), 4.0)
        ]);

        assert_eq!(aggregate.warnings.len(), 0);
    }

    #[test]
    fn test_forecast_aggregate_filter_by_date_add_warnings_for_invalid_forecasts() {
        let aggregate = ForecastAggregate::new(vec![self::create_forecast(), self::create_forecast_on_date(String::from("2019-03-05"), 4.0)], vec![])
            .filter_by_date(String::from("2019-03-02"));

        assert_eq!(aggregate.forecast_collection, vec![
            self::create_forecast_on_date(String::from("2019-03-02"), 4.0),
        ]);

        assert_eq!(aggregate.warnings.len(), 1);
    }

    #[test]
    fn test_forecast_aggregate_into_week_work_as_expected() {
        let aggregate = ForecastAggregate::new(vec![self::create_forecast(), self::create_forecast()], vec![])
            .into_week_aggregate();

        assert_eq!(aggregate.forecast_collection, vec![
            Forecast::new(
                vec![(String::from("2019-03-01"), 3.0), (String::from("2019-03-02"), 4.0), (String::from("2019-03-03"), 5.0), (String::from("2019-03-04"), 6.0), (String::from("2019-03-05"), 7.0)].into_iter().collect::<RawForecast>(),
                String::from("test"),
            ),
            Forecast::new(
                vec![(String::from("2019-03-01"), 3.0), (String::from("2019-03-02"), 4.0), (String::from("2019-03-03"), 5.0), (String::from("2019-03-04"), 6.0), (String::from("2019-03-05"), 7.0)].into_iter().collect::<RawForecast>(),
                String::from("test"),
            )
        ]);

        assert_eq!(aggregate.warnings.len(), 0);
    }

    #[test]
    fn test_forecast_aggregate_into_week_add_warnings_for_invalid_forecasts() {
        let aggregate = ForecastAggregate::new(vec![self::create_forecast(), self::create_forecast_on_date(String::from("2019-03-05"), 4.0)], vec![])
            .into_week_aggregate();

        assert_eq!(aggregate.forecast_collection, vec![
            Forecast::new(
                vec![(String::from("2019-03-01"), 3.0), (String::from("2019-03-02"), 4.0), (String::from("2019-03-03"), 5.0), (String::from("2019-03-04"), 6.0), (String::from("2019-03-05"), 7.0)].into_iter().collect::<RawForecast>(),
                String::from("test"),
            )
        ]);

        assert_eq!(aggregate.warnings.len(), 1);
    }

    #[test]
    fn test_forecast_aggregate_average_forecast() {
        let aggregate = ForecastAggregate::new(vec![self::create_forecast(), self::create_forecast()], vec![]);

        assert_eq!(aggregate.calculate_average_forecast().unwrap().items, self::create_forecast().items);
    }

    #[test]
    fn test_forecast_aggregate_average_forecast_2() {
        let aggregate = ForecastAggregate::new(vec![
            self::create_forecast(),
            Forecast::new(
                vec![(String::from("2019-03-01"), -3.0), (String::from("2019-03-02"), -4.0), (String::from("2019-03-03"), -5.0), (String::from("2019-03-04"), -6.0), (String::from("2019-03-05"), -7.0), (String::from("2019-03-06"), -8.0), (String::from("2019-03-07"), -9.0)].into_iter().collect::<RawForecast>(),
                String::from("test"),
            )
        ], vec![]);

        assert_eq!(
            aggregate.calculate_average_forecast().unwrap().items,
            Forecast::new(
                vec![(String::from("2019-03-01"), 0.0), (String::from("2019-03-02"), 0.0), (String::from("2019-03-03"), 0.0), (String::from("2019-03-04"), 0.0), (String::from("2019-03-05"), 0.0), (String::from("2019-03-06"), 0.0), (String::from("2019-03-07"), 0.0)].into_iter().collect::<RawForecast>(),
                String::from("test"),
            ).items
        );
    }


    #[test]
    fn test_forecast_aggregate_average_forecast_return_none_if_no_forecasts() {
        let aggregate = ForecastAggregate::new(vec![], vec![]);

        assert!(aggregate.calculate_average_forecast().is_none());
    }
}

#[cfg(test)]
mod forecast_test {
    use crate::app::forecast::Forecast;
    use crate::app::RawForecast;

    fn create_forecast() -> Forecast {
        Forecast::new(
            vec![(String::from("2019-03-01"), 3.0), (String::from("2019-03-02"), 4.0), (String::from("2019-03-03"), 5.0), (String::from("2019-03-04"), 6.0), (String::from("2019-03-05"), 7.0), (String::from("2019-03-06"), 8.0), (String::from("2019-03-07"), 9.0)].into_iter().collect::<RawForecast>(),
            String::from("test"),
        )
    }

    #[test]
    fn test_forecast_into_date_work_as_expected() {
        let forecast = self::create_forecast();

        let date_forecast = forecast.into_date_forecast(String::from("2019-03-02"));
        assert_eq!(date_forecast.unwrap().items, vec![(String::from("2019-03-02"), 4.0)].into_iter().collect::<RawForecast>());
    }

    #[test]
    fn test_forecast_into_date_fail_if_no_date() {
        let forecast = self::create_forecast();

        let date_forecast = forecast.into_date_forecast(String::from("2019-03-11"));
        assert!(date_forecast.is_err());
    }

    #[test]
    fn test_forecast_into_week_work_as_expected() {
        let forecast = self::create_forecast();

        let week_forecast = forecast.into_week_forecast();
        assert_eq!(week_forecast.unwrap().items, vec![(String::from("2019-03-01"), 3.0), (String::from("2019-03-02"), 4.0), (String::from("2019-03-03"), 5.0), (String::from("2019-03-04"), 6.0), (String::from("2019-03-05"), 7.0)].into_iter().collect::<RawForecast>());
    }


    #[test]
    fn test_forecast_into_week_fail_if_forecast_to_short() {
        let forecast = Forecast::new(
            vec![(String::from("2019-03-01"), 3.0), (String::from("2019-03-02"), 4.0), (String::from("2019-03-03"), 5.0)].into_iter().collect::<RawForecast>(),
            String::from("test"),
        );

        let week_forecast = forecast.into_week_forecast();
        assert!(week_forecast.is_err());
    }
}