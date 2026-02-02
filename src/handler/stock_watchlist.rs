use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::api_models::stock_watchlist::{
    AddWatchlistRequest, BatchCheckWatchlistRequest, BatchCheckWatchlistResponse,
    CheckWatchlistResponse, WatchlistResponse,
};
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::models::NewStockWatchlist;
use crate::repositories::stock_watchlist;

impl From<crate::models::StockWatchlist> for WatchlistResponse {
    fn from(item: crate::models::StockWatchlist) -> Self {
        Self {
            id: item.id,
            stock_code: item.stock_code,
            stock_name: item.stock_name,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

/// 添加股票到观察表
pub async fn add_to_watchlist(
    State(state): State<AppState>,
    Json(payload): Json<AddWatchlistRequest>,
) -> Result<(StatusCode, Json<WatchlistResponse>), AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;

    // 检查是否已存在
    let exists = stock_watchlist::exists_by_code(&mut conn, &payload.stock_code)
        .map_err(|e| {
            tracing::error!("Failed to check watchlist existence: {}", e);
            AppError::InternalServerError
        })?;

    if exists {
        // 如果已存在，返回现有记录
        let existing = stock_watchlist::find_by_code(&mut conn, &payload.stock_code)
            .map_err(|e| {
                tracing::error!("Failed to find watchlist item: {}", e);
                AppError::InternalServerError
            })?
            .ok_or(AppError::InternalServerError)?;
        return Ok((StatusCode::OK, Json(existing.into())));
    }

    let new_item = NewStockWatchlist {
        stock_code: payload.stock_code,
        stock_name: payload.stock_name,
    };

    let created = stock_watchlist::create(&mut conn, &new_item).map_err(|e| {
        tracing::error!("Failed to create watchlist item: {}", e);
        AppError::InternalServerError
    })?;

    Ok((StatusCode::CREATED, Json(created.into())))
}

/// 从观察表移除股票
pub async fn remove_from_watchlist(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<StatusCode, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;

    let affected = stock_watchlist::delete_by_code(&mut conn, &code).map_err(|e| {
        tracing::error!("Failed to delete watchlist item: {}", e);
        AppError::InternalServerError
    })?;

    if affected == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

/// 检查股票是否在观察表中
pub async fn check_watchlist(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<CheckWatchlistResponse>, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;

    let is_watched = stock_watchlist::exists_by_code(&mut conn, &code).map_err(|e| {
        tracing::error!("Failed to check watchlist: {}", e);
        AppError::InternalServerError
    })?;

    Ok(Json(CheckWatchlistResponse {
        is_watched,
        stock_code: code,
    }))
}

/// 批量检查股票是否在观察表中
#[axum::debug_handler]
pub async fn batch_check_watchlist(
    State(state): State<AppState>,
    Json(payload): Json<BatchCheckWatchlistRequest>,
) -> Result<Json<BatchCheckWatchlistResponse>, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;

    let mut watched_codes = Vec::new();

    for code in payload.stock_codes {
        let is_watched = stock_watchlist::exists_by_code(&mut conn, &code).map_err(|e| {
            tracing::error!("Failed to check watchlist for {}: {}", code, e);
            AppError::InternalServerError
        })?;

        if is_watched {
            watched_codes.push(code);
        }
    }

    Ok(Json(BatchCheckWatchlistResponse { watched_codes }))
}

/// 获取所有观察的股票
pub async fn list_watchlist(
    State(state): State<AppState>,
) -> Result<Json<Vec<WatchlistResponse>>, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;

    let items = stock_watchlist::list_all(&mut conn).map_err(|e| {
        tracing::error!("Failed to list watchlist: {}", e);
        AppError::InternalServerError
    })?;

    let response: Vec<WatchlistResponse> = items.into_iter().map(Into::into).collect();
    Ok(Json(response))
}
