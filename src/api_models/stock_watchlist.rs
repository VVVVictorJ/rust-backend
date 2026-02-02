use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AddWatchlistRequest {
    pub stock_code: String,
    pub stock_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WatchlistResponse {
    pub id: i32,
    pub stock_code: String,
    pub stock_name: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct CheckWatchlistResponse {
    pub is_watched: bool,
    pub stock_code: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BatchCheckWatchlistRequest {
    pub stock_codes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BatchCheckWatchlistResponse {
    pub watched_codes: Vec<String>,
}
