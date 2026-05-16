use axum::{routing::post, Router};

use crate::app::AppState;
use crate::handler::stock_watchlist_query::{
    fill_watchlist_klines, query_stock_detail, query_stock_kline, query_watchlist_stocks,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(query_watchlist_stocks))
        .route("/detail", post(query_stock_detail))
        .route("/kline", post(query_stock_kline))
        .route("/fill-klines", post(fill_watchlist_klines))
}
