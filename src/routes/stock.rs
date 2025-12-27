use crate::handler::stock::get_stock;
use crate::handler::stock_persist::get_filtered_stocks_param_with_persist;
use axum::{routing::get, Router};
use serde_json::{json, Value};

use crate::app::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct StockQuery {
    pub code: String,
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default)]
    pub raw_only: bool,
}

fn default_source() -> String {
    "em".to_string()
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/stock", get(get_stock))
        .route("/stock/filtered/param", get(get_filtered_stocks_param_with_persist))
}

pub fn internal_error<E: std::error::Error>(err: E) -> (axum::http::StatusCode, axum::Json<Value>) {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        axum::Json(json!({"error": "internal error", "message": err.to_string()})),
    )
}
