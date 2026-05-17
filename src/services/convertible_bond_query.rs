use rand::Rng;
use reqwest::header::HeaderMap;
use reqwest::{Client, Url};
use serde_json::Value;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use chrono::{Datelike, Local, NaiveDate, NaiveDateTime};

use crate::api_models::convertible_bond_query::ConvertibleBondItem;
use crate::utils::proxy::{proxy_get_json, shared_proxy_client, ProxyClient, ProxyError};

/// 数据中心可转债列表 GET 路径（与东方财富 WEB 客户端一致）。
const EM_CB_DATACENTER_GET: &str = "https://datacenter-web.eastmoney.com/api/data/v1/get";

#[derive(Debug, Error)]
pub enum ConvertibleBondError {
    #[error("proxy error: {0}")]
    Proxy(#[from] ProxyError),
    #[error("missing result.data")]
    MissingResultData,
    #[error("url parse error: {0}")]
    Url(String),
}

fn build_convertible_list_url(page_number: &str, page_size: &str) -> Result<Url, ConvertibleBondError> {
    let sort_columns = "PUBLIC_START_DATE,SECURITY_CODE";
    let sort_types = "-1,-1";
    let report_name = "RPT_BOND_CB_LIST";
    let columns = "ALL";
    let quote_columns = "f2~01~CONVERT_STOCK_CODE~CONVERT_STOCK_PRICE,f235~10~SECURITY_CODE~TRANSFER_PRICE,f236~10~SECURITY_CODE~TRANSFER_VALUE,f2~10~SECURITY_CODE~CURRENT_BOND_PRICE,f237~10~SECURITY_CODE~TRANSFER_PREMIUM_RATIO,f239~10~SECURITY_CODE~RESALE_TRIG_PRICE,f240~10~SECURITY_CODE~REDEEM_TRIG_PRICE,f23~01~CONVERT_STOCK_CODE~PBV_RATIO";
    let quote_type = "0";
    let source = "WEB";
    let query_client = "WEB";

    Url::parse_with_params(
        EM_CB_DATACENTER_GET,
        [
            ("sortColumns", sort_columns),
            ("sortTypes", sort_types),
            ("pageSize", page_size),
            ("pageNumber", page_number),
            ("reportName", report_name),
            ("columns", columns),
            ("quoteColumns", quote_columns),
            ("quoteType", quote_type),
            ("source", source),
            ("client", query_client),
        ],
    )
    .map_err(|e| ConvertibleBondError::Url(e.to_string()))
}

/// 接口返回的总页数；缺省时用 count 与 pageSize 换算。
fn resolve_total_pages(root: &Value, page_size_str: &str) -> i64 {
    if let Some(p) = root.pointer("/result/pages").and_then(Value::as_i64) {
        if p >= 1 {
            return p;
        }
    }

    let count = root
        .pointer("/result/count")
        .and_then(Value::as_i64)
        .unwrap_or(0)
        .max(0);
    let ps = page_size_str.parse::<i64>().unwrap_or(500).max(1);
    ((count + ps - 1) / ps).max(1)
}

async fn fetch_convertible_page_json(
    proxy_client: &Arc<Mutex<ProxyClient>>,
    headers: &HeaderMap,
    url: Url,
    page_number_one_based: i64,
) -> Result<Value, ConvertibleBondError> {
    let mut attempt = 0;
    let max_attempts = 3;
    loop {
        attempt += 1;
        match proxy_get_json(proxy_client, url.clone(), headers).await {
            Ok(json) => return Ok(json),
            Err(e) => {
                if attempt < max_attempts {
                    let backoff = 200_u64.saturating_mul(attempt as u64);
                    let jitter = rand::thread_rng().gen_range(0..=150);
                    tracing::warn!(
                        "可转债数据中心请求失败，准备重试: page={}, error={}, attempt={}",
                        page_number_one_based,
                        e,
                        attempt
                    );
                    sleep(Duration::from_millis(backoff + jitter)).await;
                    continue;
                }
                return Err(e.into());
            }
        }
    }
}

fn append_rows_from_result(all: &mut Vec<Value>, root: &Value) {
    if let Some(arr) = root.pointer("/result/data").and_then(Value::as_array) {
        all.extend(arr.iter().cloned());
    }
}

fn filter_rows_to_items(rows: &[Value]) -> Vec<ConvertibleBondItem> {
    let mut items = Vec::new();
    let now = Local::now();
    let now_year = now.year();
    let now_month = now.month();

    for row in rows {
        if let Some((y, m)) = parse_transfer_end_year_month(row) {
            if y == now_year && m == now_month {
                continue;
            }
        }

        let transfer_price = parse_number(row.get("TRANSFER_PRICE"));
        if transfer_price.is_none() {
            continue;
        }

        let issue_scale = match parse_number(row.get("ACTUAL_ISSUE_SCALE")) {
            Some(v) => v,
            None => continue,
        };
        if !(3.0..=5.0).contains(&issue_scale) {
            continue;
        }

        let transfer_premium_ratio = match parse_number(row.get("TRANSFER_PREMIUM_RATIO")) {
            Some(v) => v,
            None => continue,
        };
        if transfer_premium_ratio > 10.0 {
            continue;
        }

        let bond_price = match parse_number(row.get("CURRENT_BOND_PRICE")) {
            Some(v) => v,
            None => continue,
        };

        items.push(ConvertibleBondItem {
            bond_code: parse_string(row.get("SECURITY_CODE")),
            bond_short_name: parse_string(row.get("SECURITY_NAME_ABBR")),
            stock_code: parse_string(row.get("CONVERT_STOCK_CODE")),
            stock_name: parse_string(row.get("SECURITY_SHORT_NAME")),
            issue_scale,
            transfer_premium_ratio,
            stock_price: parse_number(row.get("CONVERT_STOCK_PRICE")),
            bond_price: Some(bond_price),
        });
    }

    items
}

pub async fn fetch_filtered_convertible_bonds(
    _client: &Client,
) -> Result<Vec<ConvertibleBondItem>, ConvertibleBondError> {
    let proxy_client = shared_proxy_client()?;

    fetch_filtered_convertible_bonds_with_proxy_client(proxy_client, _client).await
}

pub async fn fetch_filtered_convertible_bonds_with_proxy_client(
    proxy_client: Arc<Mutex<ProxyClient>>,
    _client: &Client,
) -> Result<Vec<ConvertibleBondItem>, ConvertibleBondError> {
    let headers = HeaderMap::new();
    let page_size = "500";

    let url_first = build_convertible_list_url("1", page_size)?;
    let root_first =
        fetch_convertible_page_json(&proxy_client, &headers, url_first, 1).await?;

    root_first
        .pointer("/result/data")
        .and_then(Value::as_array)
        .ok_or(ConvertibleBondError::MissingResultData)?;

    let total_pages = resolve_total_pages(&root_first, page_size);
    let mut all_rows: Vec<Value> = Vec::new();
    append_rows_from_result(&mut all_rows, &root_first);

    for pn in 2..=total_pages {
        let url = build_convertible_list_url(&pn.to_string(), page_size)?;
        let root =
            fetch_convertible_page_json(&proxy_client, &headers, url, pn).await?;
        append_rows_from_result(&mut all_rows, &root);
    }

    let market_count = root_first
        .pointer("/result/count")
        .and_then(Value::as_i64)
        .unwrap_or(all_rows.len() as i64);
    let fetched = all_rows.len();
    let items = filter_rows_to_items(&all_rows);
    tracing::info!(
        target: "convertible_bond",
        market_count,
        fetched_rows = fetched,
        matched = items.len(),
        pages = total_pages,
        "可转债查询: 数据中心约 {} 支(共 {} 页), 合并拉取 {} 条, 筛选后 {} 支",
        market_count,
        total_pages,
        fetched,
        items.len()
    );

    Ok(items)
}

fn parse_string(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or_default()
        .to_string()
}

/// 转股结束年月；字段缺失或无法解析时返回 `None`（视为不参与「当月结束」判定，条目保留）。
fn parse_transfer_end_year_month(row: &Value) -> Option<(i32, u32)> {
    match row.get("TRANSFER_END_DATE")? {
        Value::Null => None,
        Value::String(text) => {
            let cleaned = text.trim();
            if cleaned.is_empty() || cleaned == "-" || cleaned.eq_ignore_ascii_case("null") {
                return None;
            }
            NaiveDateTime::parse_from_str(cleaned, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|dt| (dt.year(), dt.month()))
                .or_else(|| {
                    NaiveDate::parse_from_str(cleaned, "%Y-%m-%d")
                        .ok()
                        .map(|d| (d.year(), d.month()))
                })
        }
        Value::Number(_) => None,
        _ => None,
    }
}

fn parse_number(value: Option<&Value>) -> Option<f64> {
    let value = value?;
    match value {
        Value::Number(num) => num.as_f64(),
        Value::String(text) => {
            let cleaned = text.trim();
            if cleaned.is_empty() || cleaned == "-" || cleaned.eq_ignore_ascii_case("null") {
                return None;
            }
            cleaned.parse::<f64>().ok()
        }
        _ => None,
    }
}
