use axum::{extract::State, Json};

use crate::api_models::stock_appearance_query::{
    StockAppearanceQueryItem, StockAppearanceQueryRequest, StockAppearanceQueryResponse,
};
use crate::api_models::stock_trade_date_query::PlateInfo;
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::repositories::stock_appearance_query;

/// 按股票代码/股票名称/板块查询股票快照历史出现记录
pub async fn query_appearances(
    State(state): State<AppState>,
    Json(payload): Json<StockAppearanceQueryRequest>,
) -> Result<Json<StockAppearanceQueryResponse>, AppError> {
    // 验证分页参数
    if payload.page < 1 {
        return Err(AppError::BadRequest(
            "page must be greater than 0".to_string(),
        ));
    }
    if payload.page_size < 1 || payload.page_size > 100 {
        return Err(AppError::BadRequest(
            "page_size must be between 1 and 100".to_string(),
        ));
    }

    // 归一化查询条件：空白字符串视为未填写
    let stock_code = payload
        .stock_code
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let stock_name = payload
        .stock_name
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let plate_code = payload
        .plate_code
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    if stock_code.is_none() && stock_name.is_none() && plate_code.is_none() {
        return Err(AppError::BadRequest(
            "at least one query condition is required".to_string(),
        ));
    }

    // 获取数据库连接
    let mut conn = state
        .db_pool
        .get()
        .map_err(|_| AppError::InternalServerError)?;

    // 计算分页参数
    let offset = (payload.page - 1) * payload.page_size;

    // 查询总数
    let total = stock_appearance_query::count_by_condition(
        &mut conn,
        stock_code.clone(),
        stock_name.clone(),
        plate_code.clone(),
    )
    .map_err(|e| {
        tracing::error!("Failed to count appearance records: {}", e);
        AppError::InternalServerError
    })?;

    // 查询数据
    let results = stock_appearance_query::query_by_condition(
        &mut conn,
        stock_code,
        stock_name,
        plate_code,
        payload.page_size,
        offset,
    )
    .map_err(|e| {
        tracing::error!("Failed to query appearance records: {}", e);
        AppError::InternalServerError
    })?;

    // 转换结果
    let data = results
        .into_iter()
        .map(|r| {
            let plates: Vec<PlateInfo> = serde_json::from_value(r.plates).unwrap_or_default();

            StockAppearanceQueryItem {
                stock_code: r.stock_code,
                stock_name: r.stock_name,
                latest_price: r.latest_price,
                close_price: r.close_price,
                change_pct: r.change_pct,
                volume_ratio: r.volume_ratio,
                turnover_rate: r.turnover_rate,
                bid_ask_ratio: r.bid_ask_ratio,
                main_force_inflow: r.main_force_inflow,
                created_at: r.created_at,
                plates,
            }
        })
        .collect();

    // 计算总页数
    let total_pages = if total == 0 {
        0
    } else {
        (total + payload.page_size - 1) / payload.page_size
    };

    Ok(Json(StockAppearanceQueryResponse {
        data,
        total,
        page: payload.page,
        page_size: payload.page_size,
        total_pages,
    }))
}
