use axum::{routing::get, Json, Router};
use serde::Serialize;

pub fn router() -> Router {
    Router::new().route("/stock", get(get_stock))
}

#[derive(Serialize)]
struct StockResponse<'a> {
    symbol: &'a str,
    name:   &'a str,
    price:  f64,
}

async fn get_stock() -> Json<StockResponse<'static>> {
    // mock 数据
    Json(StockResponse {
        symbol: "AAPL",
        name: "Apple Inc.",
        price: 199.88,
    })
}