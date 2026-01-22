use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde_json::Value;

use crate::app::AppState;
use crate::handler::stock::FilterParamQuery;
use crate::models::{NewStockRequest, NewStockSnapshot};
use crate::repositories::{stock_request, stock_snapshot};
use crate::routes::stock::internal_error;
use crate::services::stock_filter::{get_filtered_stocks_param as svc_get_filtered_stocks_param, FilterParams};
use crate::utils::bigdecimal_parser::parse_bigdecimal;

/// 带数据库持久化的筛选股票接口
/// 在获取筛选结果后，如果 items 非空，自动将请求和快照数据存入数据库
pub async fn get_filtered_stocks_param_with_persist(
    State(state): State<AppState>,
    Query(p): Query<FilterParamQuery>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // 构建筛选参数
    let params = FilterParams {
        pct_min: p.pct_min,
        pct_max: p.pct_max,
        lb_min: p.lb_min,
        hs_min: p.hs_min,
        wb_min: p.wb_min,
        concurrency: p.concurrency.clamp(1, 64) as usize,
        limit: p.limit.max(0) as usize,
        pz: p.pz,
    };

    // 构建 HTTP 客户端
    let client = {
        use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_ENCODING, REFERER, USER_AGENT};
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            ),
        );
        headers.insert(ACCEPT, HeaderValue::from_static("application/json, text/plain, */*"));
        headers.insert(REFERER, HeaderValue::from_static("https://quote.eastmoney.com"));
        headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip"));
        reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap()
    };

    // 调用现有服务函数获取筛选结果
    let result = match svc_get_filtered_stocks_param(&client, params).await {
        Ok(r) => r,
        Err(e) => {
            // 外部 API 请求失败，返回 503 Service Unavailable
            let err_msg = e.to_string();
            let is_network_error = err_msg.contains("http error") || err_msg.contains("error sending request");
            if is_network_error {
                return Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(serde_json::json!({
                        "error": "upstream_unavailable",
                        "message": "无法连接东方财富 API，请稍后重试或检查网络",
                        "detail": err_msg
                    })),
                ));
            }
            return Err(internal_error(e));
        }
    };

    // 检查 items 是否非空
    let items = result.get("items").and_then(|v| v.as_array());
    if let Some(items_arr) = items {
        if !items_arr.is_empty() {
            // 尝试持久化到数据库（失败不影响 API 返回）
            if let Err(e) = persist_to_db(&state, items_arr).await {
                tracing::warn!("Failed to persist stock data: {}", e);
            }
        }
    }

    Ok((StatusCode::OK, Json(result)))
}

/// 将筛选结果持久化到数据库
async fn persist_to_db(state: &AppState, items: &[Value]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = state.db_pool.get()?;

    // 1. 插入 stock_requests 记录
    let now_date = chrono::Utc::now().date_naive();
    let new_request = NewStockRequest {
        strategy_name: Some("filtered_param".to_string()),
        time_range_start: Some(now_date),
        time_range_end: None,  // 待处理，收益分析完成后才设置
    };
    let created_request = stock_request::create(&mut conn, &new_request)?;
    let request_id = created_request.id;

    // 2. 遍历 items，插入 stock_snapshots
    for item in items {
        let stock_code = item
            .get("f57")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let stock_name = item
            .get("f58")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let latest_price = parse_bigdecimal(item.get("f43"));
        let change_pct = parse_bigdecimal(item.get("f170"));
        let volume_ratio = parse_bigdecimal(item.get("f50"));
        let turnover_rate = parse_bigdecimal(item.get("f168"));
        let bid_ask_ratio = parse_bigdecimal(item.get("f191"));
        let main_force_inflow = parse_bigdecimal(item.get("f137"));

        let new_snapshot = NewStockSnapshot {
            request_id,
            stock_code,
            stock_name,
            latest_price,
            change_pct,
            volume_ratio,
            turnover_rate,
            bid_ask_ratio,
            main_force_inflow,
        };

        if let Err(e) = stock_snapshot::create(&mut conn, &new_snapshot) {
            tracing::warn!("Failed to insert snapshot: {}", e);
        }
    }

    Ok(())
}
