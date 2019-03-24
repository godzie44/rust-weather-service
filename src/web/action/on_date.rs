use actix_web::HttpRequest;
use actix_web::FutureResponse;
use actix_web::HttpResponse;
use futures::Future;
use crate::app::provider::apixu;
use crate::app::provider::yahoo;
use crate::app::WeatherAggregateManager;
use crate::web::AppState;

use actix_web::error;
use crate::web::action::ForecastUserResponse;

pub fn handle(req: &HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let location = req.match_info().get("location").unwrap();
    let date = req.match_info().get("date").unwrap();

    let config = req.state().config.lock().unwrap();

    let aggregate_manager = WeatherAggregateManager::new(vec![
        Box::new(apixu::ApixuProvider::new(config.get("apixu_key").unwrap().clone())),
        Box::new(yahoo::YahooProvider::new(
            config.get("yahoo_app_id").unwrap().clone(),
            config.get("yahoo_secret").unwrap().clone(),
            config.get("yahoo_user_key").unwrap().clone(),
        )),
    ]);

    Box::new(
        aggregate_manager
            .get_forecast_aggregate_at(date.to_owned(), location)
            .and_then(|forecast_aggregate| {
                let aggregate_result = forecast_aggregate.calculate_average_forecast();
                Ok(HttpResponse::Ok().json(ForecastUserResponse {
                    ok: aggregate_result.is_some(),
                    forecast: aggregate_result,
                    warnings: forecast_aggregate.get_warnings().clone()
                }))

            })
            .map_err(|e| error::ErrorBadRequest(e))
    )
}

