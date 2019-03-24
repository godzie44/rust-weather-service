use crate::app::forecast::Forecast;

pub mod on_week;
pub mod on_date;

#[derive(Debug, Serialize, Deserialize)]
pub struct ForecastUserResponse {
    pub ok: bool,
    pub forecast: Option<Forecast>,
    pub warnings: Vec<String>
}