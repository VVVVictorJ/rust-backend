use rand::Rng;
use reqwest::header::HeaderMap;
use reqwest::{Client, Url};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use chrono::{Local, NaiveDate, NaiveDateTime};
use futures::stream::{self, StreamExt};

use crate::api_models::convertible_bond_query::ConvertibleBondItem;
use crate::utils::proxy::{proxy_get_json, shared_proxy_client, ProxyClient, ProxyError};

/// 数据中心可转债列表 GET 路径（与东方财富 WEB 客户端一致）。
const EM_CB_DATACENTER_GET: &str = "https://datacenter-web.eastmoney.com/api/data/v1/get";

/// DELIST_DATE 非空债券并发请求「重要日期」接口的并行度。
const IMPORTANT_DATE_CONCURRENCY: usize = 8;

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

/// `RPT_CB_IMPORTANTDATE`，filter 形如 `(SECURITY_CODE="118004")`。
fn build_cb_important_date_url(security_code: &str) -> Result<Url, ConvertibleBondError> {
    let millis = Local::now().timestamp_millis().to_string();
    let filter = format!("(SECURITY_CODE=\"{security_code}\")");
    Url::parse_with_params(
        EM_CB_DATACENTER_GET,
        [
            ("reportName", "RPT_CB_IMPORTANTDATE"),
            ("columns", "ALL"),
            ("quoteColumns", ""),
            ("source", "WEB"),
            ("client", "WEB"),
            ("filter", filter.as_str()),
            ("_", millis.as_str()),
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

async fn fetch_important_date_json_with_retry(
    proxy_client: &Arc<Mutex<ProxyClient>>,
    headers: &HeaderMap,
    url: Url,
    bond_code: &str,
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
                        target: "convertible_bond",
                        "可转债重要日期请求失败，准备重试: code={}, error={}, attempt={}",
                        bond_code,
                        e,
                        attempt
                    );
                    sleep(Duration::from_millis(backoff + jitter)).await;
                    continue;
                }
                tracing::warn!(
                    target: "convertible_bond",
                    "可转债重要日期请求失败，放弃: code={}, error={}",
                    bond_code,
                    e
                );
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

#[derive(Clone)]
struct NumericCbFields {
    issue_scale: f64,
    transfer_premium_ratio: f64,
    bond_price: f64,
}

fn parse_numeric_cb_fields(row: &Value) -> Option<NumericCbFields> {
    parse_number(row.get("TRANSFER_PRICE"))?;

    let issue_scale = parse_number(row.get("ACTUAL_ISSUE_SCALE"))?;
    if !(3.0..=5.0).contains(&issue_scale) {
        return None;
    }

    let transfer_premium_ratio = parse_number(row.get("TRANSFER_PREMIUM_RATIO"))?;
    if transfer_premium_ratio > 10.0 {
        return None;
    }

    let bond_price = parse_number(row.get("CURRENT_BOND_PRICE"))?;

    Some(NumericCbFields {
        issue_scale,
        transfer_premium_ratio,
        bond_price,
    })
}

fn build_convertible_item(row: &Value, fields: &NumericCbFields, near_last_trading_day: bool) -> ConvertibleBondItem {
    ConvertibleBondItem {
        bond_code: parse_string(row.get("SECURITY_CODE")),
        bond_short_name: parse_string(row.get("SECURITY_NAME_ABBR")),
        stock_code: parse_string(row.get("CONVERT_STOCK_CODE")),
        stock_name: parse_string(row.get("SECURITY_SHORT_NAME")),
        issue_scale: fields.issue_scale,
        transfer_premium_ratio: fields.transfer_premium_ratio,
        stock_price: parse_number(row.get("CONVERT_STOCK_PRICE")),
        bond_price: Some(fields.bond_price),
        near_last_trading_day,
    }
}

/// 解析重要日期接口中「最后交易日」的 START_DATE。
fn parse_last_trading_day_start(root: &Value) -> Option<NaiveDate> {
    let arr = root.pointer("/result/data")?.as_array()?;
    for row in arr {
        let dtype = parse_string(row.get("DATE_TYPE"));
        if dtype != "最后交易日" {
            continue;
        }
        return parse_em_date_field(row.get("START_DATE"));
    }
    None
}

fn parse_em_date_field(value: Option<&Value>) -> Option<NaiveDate> {
    match value? {
        Value::Null => None,
        Value::String(text) => {
            let cleaned = text.trim();
            if cleaned.is_empty() || cleaned == "-" || cleaned.eq_ignore_ascii_case("null") {
                return None;
            }
            NaiveDateTime::parse_from_str(cleaned, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|dt| dt.date())
                .or_else(|| NaiveDate::parse_from_str(cleaned, "%Y-%m-%d").ok())
        }
        _ => None,
    }
}

async fn fetch_last_trade_dates_by_codes(
    proxy_client: Arc<Mutex<ProxyClient>>,
    headers: HeaderMap,
    codes: &[String],
) -> HashMap<String, NaiveDate> {
    let mut map = HashMap::with_capacity(codes.len());
    if codes.is_empty() {
        return map;
    }

    let fetched: Vec<(String, Option<NaiveDate>)> = stream::iter(codes.iter().cloned())
        .map(|code| {
            let proxy = proxy_client.clone();
            let h = headers.clone();
            async move {
                let url = match build_cb_important_date_url(&code) {
                    Ok(u) => u,
                    Err(e) => {
                        tracing::warn!(
                            target: "convertible_bond",
                            "构建重要日期 URL 失败 code={}: {}",
                            code,
                            e
                        );
                        return (code, None);
                    }
                };
                match fetch_important_date_json_with_retry(&proxy, &h, url, &code).await {
                    Ok(json) => {
                        let d = parse_last_trading_day_start(&json);
                        if d.is_none() {
                            tracing::warn!(
                                target: "convertible_bond",
                                "重要日期无「最后交易日」或未解析 START_DATE: code={}",
                                code
                            );
                        }
                        (code, d)
                    }
                    Err(_) => (code, None),
                }
            }
        })
        .buffer_unordered(IMPORTANT_DATE_CONCURRENCY)
        .collect()
        .await;

    for (code, opt) in fetched {
        if let Some(d) = opt {
            map.insert(code, d);
        }
    }
    map
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

    let today = Local::now().date_naive();

    let mut plain_items: Vec<ConvertibleBondItem> = Vec::new();
    let mut delist_rows: Vec<Value> = Vec::new();

    for row in &all_rows {
        let Some(fields) = parse_numeric_cb_fields(row) else {
            continue;
        };
        if delist_date_is_nonempty(row) {
            delist_rows.push(row.clone());
        } else {
            plain_items.push(build_convertible_item(row, &fields, false));
        }
    }

    let mut codes_unique: Vec<String> = delist_rows
        .iter()
        .map(|r| parse_string(r.get("SECURITY_CODE")))
        .filter(|c| !c.is_empty())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    codes_unique.sort();

    let last_trade_map =
        fetch_last_trade_dates_by_codes(proxy_client.clone(), headers.clone(), &codes_unique).await;

    let mut delist_near_items: Vec<ConvertibleBondItem> = Vec::new();
    for row in delist_rows {
        let Some(fields) = parse_numeric_cb_fields(&row) else {
            continue;
        };
        let code = parse_string(row.get("SECURITY_CODE"));
        let Some(last_start) = last_trade_map.get(&code).copied() else {
            continue;
        };
        let days = last_start.signed_duration_since(today).num_days();
        if !(0..=3).contains(&days) {
            continue;
        }
        delist_near_items.push(build_convertible_item(&row, &fields, true));
    }

    let matched_len = plain_items.len() + delist_near_items.len();
    plain_items.extend(delist_near_items);

    let market_count = root_first
        .pointer("/result/count")
        .and_then(Value::as_i64)
        .unwrap_or(all_rows.len() as i64);
    let fetched = all_rows.len();
    tracing::info!(
        target: "convertible_bond",
        market_count,
        fetched_rows = fetched,
        matched = matched_len,
        pages = total_pages,
        "可转债查询: 数据中心约 {} 支(共 {} 页), 合并拉取 {} 条, 筛选后 {} 支",
        market_count,
        total_pages,
        fetched,
        matched_len,
    );

    Ok(plain_items)
}

fn parse_string(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or_default()
        .to_string()
}

/// `DELIST_DATE` 视为「有摘牌日」：非空字符串且非占位符则返回 true。
fn delist_date_is_nonempty(row: &Value) -> bool {
    match row.get("DELIST_DATE") {
        None | Some(Value::Null) => false,
        Some(Value::String(s)) => {
            let t = s.trim();
            !t.is_empty() && t != "-" && !t.eq_ignore_ascii_case("null")
        }
        Some(Value::Number(_)) | Some(Value::Bool(_)) => true,
        _ => false,
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
