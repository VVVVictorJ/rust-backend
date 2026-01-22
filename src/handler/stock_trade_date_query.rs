use axum::{
    extract::State,
    Json,
};
use chrono::NaiveDate;
use serde_json;

use crate::api_models::stock_trade_date_query::{
    PlateInfo, TradeDatePlateRefreshRequest, TradeDatePlateRefreshResponse, TradeDateQueryRequest,
    TradeDateQueryItem, TradeDateQueryResponse,
};
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::models::{NewStockPlate, NewStockPlateStockTable, NewStockTable, UpdateStockPlate};
use crate::repositories::{stock_plate, stock_plate_stock_table, stock_table, stock_trade_date_query};
use crate::services::stock_plate_em::fetch_em_plate_list;
use chrono::Local;
use reqwest::Client;

/// 根据交易日期查询股票快照数据
pub async fn query_by_trade_date(
    State(state): State<AppState>,
    Json(payload): Json<TradeDateQueryRequest>,
) -> Result<Json<TradeDateQueryResponse>, AppError> {
    // 验证分页参数
    if payload.page < 1 {
        return Err(AppError::BadRequest("page must be greater than 0".to_string()));
    }
    if payload.page_size < 1 || payload.page_size > 100 {
        return Err(AppError::BadRequest("page_size must be between 1 and 100".to_string()));
    }

    // 解析交易日期
    let trade_date = NaiveDate::parse_from_str(&payload.trade_date, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid date format, expected YYYY-MM-DD".to_string()))?;

    // 获取数据库连接
    let mut conn = state
        .db_pool
        .get()
        .map_err(|_| AppError::InternalServerError)?;

    // 计算分页参数
    let offset = (payload.page - 1) * payload.page_size;

    // 查询总数
    let total = stock_trade_date_query::count_by_trade_date(&mut conn, trade_date)
        .map_err(|e| {
            tracing::error!("Failed to count records: {}", e);
            AppError::InternalServerError
        })?;

    // 查询数据
    let results = stock_trade_date_query::query_by_trade_date(
        &mut conn,
        trade_date,
        payload.page_size,
        offset,
    )
    .map_err(|e| {
        tracing::error!("Failed to query data: {}", e);
        AppError::InternalServerError
    })?;

    // 转换结果
    let data = results
        .into_iter()
        .map(|r| {
            let plates: Vec<PlateInfo> = serde_json::from_value(r.plates).unwrap_or_default();

            TradeDateQueryItem {
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

    Ok(Json(TradeDateQueryResponse {
        data,
        total,
        page: payload.page,
        page_size: payload.page_size,
        total_pages,
    }))
}

/// 补全交易日缺失的板块信息
pub async fn refresh_missing_plates(
    State(state): State<AppState>,
    Json(payload): Json<TradeDatePlateRefreshRequest>,
) -> Result<Json<TradeDatePlateRefreshResponse>, AppError> {
    let trade_date = NaiveDate::parse_from_str(&payload.trade_date, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid date format, expected YYYY-MM-DD".to_string()))?;

    let stocks = {
        let mut conn = state
            .db_pool
            .get()
            .map_err(|_| AppError::InternalServerError)?;
        stock_trade_date_query::list_stocks_by_trade_date(&mut conn, trade_date)
            .map_err(|e| {
                tracing::error!("Failed to list stocks by trade date: {}", e);
                AppError::InternalServerError
            })?
    };

    let total_count = stocks.len() as i64;
    if total_count == 0 {
        return Ok(Json(TradeDatePlateRefreshResponse {
            total_count: 0,
            missing_count: 0,
            stock_table_inserted: 0,
            plate_inserted: 0,
            plate_updated: 0,
            relation_inserted: 0,
            failed_count: 0,
        }));
    }

    let client = Client::new();
    let mut missing_count = 0_i64;
    let mut stock_table_inserted = 0_i64;
    let mut plate_inserted = 0_i64;
    let mut plate_updated = 0_i64;
    let mut relation_inserted = 0_i64;
    let mut failed_count = 0_i64;

    for stock in stocks {
        let (stock_id, has_relations) = {
            let mut conn = state
                .db_pool
                .get()
                .map_err(|_| AppError::InternalServerError)?;

            let mut stock_record = stock_table::find_by_code(&mut conn, &stock.stock_code)
                .map_err(|_| AppError::InternalServerError)?;
            if stock_record.is_none() {
                let new_stock = NewStockTable {
                    stock_code: stock.stock_code.clone(),
                    stock_name: stock.stock_name.clone(),
                };
                stock_record = Some(
                    stock_table::create(&mut conn, &new_stock)
                        .map_err(|_| AppError::InternalServerError)?,
                );
                stock_table_inserted += 1;
            }
            let stock_id = stock_record
                .as_ref()
                .map(|item| item.id)
                .unwrap_or_default();
            let relations =
                stock_plate_stock_table::list_by_stock_table_id(&mut conn, stock_id)
                    .map_err(|_| AppError::InternalServerError)?;
            (stock_id, !relations.is_empty())
        };

        if has_relations {
            continue;
        }
        missing_count += 1;

        let res = match fetch_em_plate_list(&client, &stock.stock_code).await {
            Ok(res) => res,
            Err(e) => {
                failed_count += 1;
                tracing::warn!(
                    "板块补全请求失败: stock_code={}, error={}",
                    stock.stock_code,
                    e
                );
                continue;
            }
        };

        if res.items.is_empty() {
            continue;
        }

        let mut conn = state
            .db_pool
            .get()
            .map_err(|_| AppError::InternalServerError)?;
        for item in res.items {
            let plate = if let Some(mut plate) =
                stock_plate::find_by_plate_code(&mut conn, &item.plate_code)
                    .map_err(|_| AppError::InternalServerError)?
            {
                if plate.name != item.name {
                    let update = UpdateStockPlate {
                        plate_code: None,
                        name: Some(item.name.clone()),
                        updated_at: Some(Local::now().naive_local()),
                    };
                    if let Ok(updated) = stock_plate::update_by_id(&mut conn, plate.id, &update) {
                        plate = updated;
                        plate_updated += 1;
                    }
                }
                plate
            } else if let Some(mut plate) = stock_plate::find_by_name(&mut conn, &item.name)
                .map_err(|_| AppError::InternalServerError)?
            {
                if plate.plate_code != item.plate_code || plate.name != item.name {
                    let update = UpdateStockPlate {
                        plate_code: Some(item.plate_code.clone()),
                        name: Some(item.name.clone()),
                        updated_at: Some(Local::now().naive_local()),
                    };
                    if let Ok(updated) = stock_plate::update_by_id(&mut conn, plate.id, &update) {
                        plate = updated;
                        plate_updated += 1;
                    }
                }
                plate
            } else {
                let new_plate = NewStockPlate {
                    plate_code: item.plate_code.clone(),
                    name: item.name.clone(),
                };
                match stock_plate::create(&mut conn, &new_plate) {
                    Ok(inserted) => {
                        plate_inserted += 1;
                        inserted
                    }
                    Err(e) => {
                        tracing::warn!(
                            "插入 stock_plate 失败: code={}, name={}, err={}",
                            item.plate_code,
                            item.name,
                            e
                        );
                        if let Some(existing) = stock_plate::find_by_name(&mut conn, &item.name)
                            .map_err(|_| AppError::InternalServerError)?
                        {
                            existing
                        } else {
                            continue;
                        }
                    }
                }
            };

            let exists = stock_plate_stock_table::exists_by_ids(&mut conn, plate.id, stock_id)
                .map_err(|_| AppError::InternalServerError)?;
            if !exists {
                let new_rel = NewStockPlateStockTable {
                    plate_id: plate.id,
                    stock_table_id: stock_id,
                };
                if stock_plate_stock_table::create(&mut conn, &new_rel).is_ok() {
                    relation_inserted += 1;
                }
            }
        }
    }

    Ok(Json(TradeDatePlateRefreshResponse {
        total_count,
        missing_count,
        stock_table_inserted,
        plate_inserted,
        plate_updated,
        relation_inserted,
        failed_count,
    }))
}

