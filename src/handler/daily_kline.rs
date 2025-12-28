use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::NaiveDate;
use diesel::result::Error as DieselError;

use crate::api_models::daily_kline::{CreateDailyKline, DailyKlineResponse};
use crate::api_models::kline_import::{ImportKlineRequest, ImportKlineResponse};
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::models::NewDailyKline;
use crate::repositories::daily_kline;
use crate::utils::http_client;

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

/// 创建单条 K线数据
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

/// 查询单条 K线数据
pub async fn get_daily_kline(
    State(state): State<AppState>,
    Path((code, date)): Path<(String, NaiveDate)>,
) -> Result<Json<DailyKlineResponse>, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let found = daily_kline::find_by_pk(&mut conn, &code, date).map_err(map_err)?;
    Ok(Json(found.into()))
}

/// 删除单条 K线数据
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

/// 从东方财富导入 K线数据
pub async fn kline_import(
    State(state): State<AppState>,
    Json(payload): Json<ImportKlineRequest>,
) -> Result<Json<ImportKlineResponse>, AppError> {
    // 1. 创建 HTTP 客户端
    let client = http_client::create_em_client()
        .map_err(|_| AppError::InternalServerError)?;

    // 2. 调用 service 层获取并解析数据
    let kline_result = crate::services::kline_service::fetch_and_parse_kline_data(
        &client,
        &payload.stock_code,
        &payload.start_date,
        &payload.end_date,
    )
    .await
    .map_err(|e| AppError::BadRequest(format!("Failed to fetch kline data: {}", e)))?;

    // 3. 获取数据库连接
    let mut conn = state
        .db_pool
        .get()
        .map_err(|_| AppError::InternalServerError)?;

    // 4. 批量插入数据库
    let mut imported_count = 0;
    let mut failed_count = 0;
    let mut errors = kline_result.errors.clone();

    for kline_data in kline_result.parsed {
        match daily_kline::create(&mut conn, &kline_data) {
            Ok(_) => imported_count += 1,
            Err(e) => {
                // 区分重复数据和真正的错误
                match e {
                    DieselError::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _) => {
                        errors.push(format!(
                            "Duplicate entry for {} on {}",
                            kline_data.stock_code, kline_data.trade_date
                        ));
                    }
                    _ => {
                        errors.push(format!(
                            "Failed to insert {} on {}: {}",
                            kline_data.stock_code, kline_data.trade_date, e
                        ));
                        failed_count += 1;
                    }
                }
            }
        }
    }

    // 5. 返回结果
    Ok(Json(ImportKlineResponse {
        success: failed_count == 0,
        stock_code: kline_result.stock_code,
        stock_name: kline_result.stock_name,
        total_count: kline_result.total,
        imported_count,
        failed_count,
        errors,
    }))
}

fn map_err(err: DieselError) -> AppError {
    match err {
        DieselError::NotFound => AppError::NotFound,
        DieselError::DatabaseError(_, info) => AppError::BadRequest(info.message().to_string()),
        _ => AppError::InternalServerError,
    }
}
