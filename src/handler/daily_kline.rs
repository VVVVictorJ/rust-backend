use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::NaiveDate;
use diesel::result::Error as DieselError;

use crate::api_models::daily_kline::{CreateDailyKline, DailyKlineResponse};
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::models::NewDailyKline;
use crate::repositories::daily_kline;

impl From<crate::models::DailyKline> for DailyKlineResponse {
    fn from(d: crate::models::DailyKline) -> Self {
        Self {
            stock_code: d.stock_code,
            trade_date: d.trade_date,
            open_price: d.open_price,
            high_price: d.high_price,
            low_price: d.low_price,
            close_price: d.close_price,
            volume: d.volume,
            amount: d.amount,
        }
    }
}

pub async fn create_daily_kline(
    State(state): State<AppState>,
    Json(payload): Json<CreateDailyKline>,
) -> Result<(StatusCode, Json<DailyKlineResponse>), AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let new_rec = NewDailyKline {
        stock_code: payload.stock_code,
        trade_date: payload.trade_date,
        open_price: payload.open_price,
        high_price: payload.high_price,
        low_price: payload.low_price,
        close_price: payload.close_price,
        volume: payload.volume,
        amount: payload.amount,
    };
    let created = daily_kline::create(&mut conn, &new_rec).map_err(map_err)?;
    Ok((StatusCode::CREATED, Json(created.into())))
}

pub async fn get_daily_kline(
    State(state): State<AppState>,
    Path((code, date)): Path<(String, NaiveDate)>,
) -> Result<Json<DailyKlineResponse>, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let found = daily_kline::find_by_pk(&mut conn, &code, date).map_err(map_err)?;
    Ok(Json(found.into()))
}

pub async fn delete_daily_kline(
    State(state): State<AppState>,
    Path((code, date)): Path<(String, NaiveDate)>,
) -> Result<StatusCode, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let affected = daily_kline::delete_by_pk(&mut conn, &code, date).map_err(map_err)?;
    if affected == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

fn map_err(err: DieselError) -> AppError {
    match err {
        DieselError::NotFound => AppError::NotFound,
        DieselError::DatabaseError(_, info) => AppError::BadRequest(info.message().to_string()),
        _ => AppError::InternalServerError,
    }
}

