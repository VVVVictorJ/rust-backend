use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Result;
use polars::prelude::*;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_ENCODING, REFERER, USER_AGENT};
use reqwest::{Client, Url};
use serde_json::Value;
use thiserror::Error;
use tokio::sync::Semaphore;

use crate::api_models::stock::FilteredStockItem;
use crate::utils::percent::normalize_percent_scalar;
use crate::utils::proxy::{proxy_get_json, shared_proxy_client, ProxyError};
use crate::utils::secid::code_to_secid;

const EM_LIST_URL: &str = "https://push2.eastmoney.com/api/qt/clist/get";
const EM_DETAIL_URL: &str = "https://push2.eastmoney.com/api/qt/stock/get";

#[derive(Debug, Error)]
pub enum StockFilterError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("proxy error: {0}")]
    Proxy(#[from] ProxyError),
    #[error("polars error: {0}")]
    Polars(#[from] PolarsError),
    #[error("url parse error: {0}")]
    Url(String),
}

#[derive(Debug, Clone)]
pub struct FilterParams {
    pub pct_min: f64,
    pub pct_max: f64,
    pub lb_min: f64,
    pub hs_min: f64,
    pub wb_min: f64,
    pub concurrency: usize,
    pub limit: usize,
    pub pz: i32,
}

impl Default for FilterParams {
    fn default() -> Self {
        Self {
            pct_min: 2.0,
            pct_max: 5.0,
            lb_min: 5.0,
            hs_min: 1.0,
            wb_min: 20.0,
            concurrency: 8,
            limit: 0,
            pz: 1000,
        }
    }
}

pub async fn get_filtered_stocks_param(_client: &Client, params: FilterParams) -> Result<Value, StockFilterError> {
    // clamp
    let concurrency = params.concurrency.clamp(1, 64);
    let pz = params.pz.clamp(100, 5000);
    let headers = em_headers();
    let proxy_client = shared_proxy_client()?;

    // page 1 for total and first diff
    let first_url = build_list_url(1, pz)?;
    let first = proxy_get_json(&proxy_client, first_url, &headers).await?;

    let data = first.get("data").cloned().unwrap_or(Value::Null);
    let total = data.get("total").and_then(|v| v.as_i64()).unwrap_or(0);
    let mut all = Vec::new();
    if let Some(diff) = data.get("diff").and_then(|v| v.as_array()) {
        all.extend_from_slice(diff);
    }
    let pages = if total <= 0 { 1 } else { (total as i32 + pz - 1) / pz };

    // fetch rest pages with limited concurrency
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut handles = Vec::new();
    for pn in 2..=pages {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let proxy = proxy_client.clone();
        let headers = headers.clone();
        let h = tokio::spawn(async move {
            let _p = permit;
            let url = match build_list_url(pn, pz) {
                Ok(url) => url,
                Err(_) => return None,
            };
            (proxy_get_json(&proxy, url, &headers).await).ok()
        });
        handles.push(h);
    }
    for h in handles {
        if let Ok(Some(v)) = h.await {
            if let Some(arr) = v.get("data").and_then(|d| d.get("diff")).and_then(|x| x.as_array()) {
                all.extend_from_slice(arr);
            }
        }
    }

    // build columns for polars
    let mut col_f12: Vec<Option<String>> = Vec::with_capacity(all.len());
    let mut col_f14: Vec<Option<String>> = Vec::with_capacity(all.len());
    let mut col_f15: Vec<Option<f64>> = Vec::with_capacity(all.len());
    let mut col_f3: Vec<Option<f64>> = Vec::with_capacity(all.len());
    let mut col_f10: Vec<Option<f64>> = Vec::with_capacity(all.len());
    let mut col_f8: Vec<Option<f64>> = Vec::with_capacity(all.len());
    for item in &all {
        let code = item.get("f12").and_then(|v| v.as_str()).map(|s| s.to_string());
        let name = item.get("f14").and_then(|v| v.as_str()).map(|s| s.to_string());
        let f15 = item.get("f15").and_then(|v| v.as_f64());
        let f3_v = match item.get("f3") {
            Some(Value::String(s)) => normalize_percent_scalar(s.as_str()),
            Some(Value::Number(n)) => n.as_f64(),
            _ => None,
        };
        let f10_v = match item.get("f10") {
            Some(Value::String(s)) => normalize_percent_scalar(s.as_str()),
            Some(Value::Number(n)) => n.as_f64(),
            _ => None,
        };
        let f8_v = match item.get("f8") {
            Some(Value::String(s)) => normalize_percent_scalar(s.as_str()),
            Some(Value::Number(n)) => n.as_f64(),
            _ => None,
        };
        col_f12.push(code);
        col_f14.push(name);
        col_f15.push(f15);
        col_f3.push(f3_v);
        col_f10.push(f10_v);
        col_f8.push(f8_v);
    }

    let df = DataFrame::new(vec![
        Series::new("f12", col_f12),
        Series::new("f14", col_f14),
        Series::new("f15", col_f15),
        Series::new("f3", col_f3),
        Series::new("f10", col_f10),
        Series::new("f8", col_f8),
    ])?;

    let lf = df
        .lazy()
        .filter(col("f3").gt(params.pct_min).and(col("f3").lt(params.pct_max)))
        .filter(col("f10").gt(params.lb_min))
        .filter(col("f8").gt(params.hs_min));
    let filtered = lf.collect()?;

    // collect codes
    let mut codes: Vec<String> = Vec::new();
    let mut seen = HashSet::new();
    let col_f12 = filtered.column("f12")?;
    let col_f12_str = col_f12.str()?;
    for s in col_f12_str.into_iter().flatten() {
        let code = s.to_string();
        if seen.insert(code.clone()) {
            codes.push(code);
        }
    }

    // fetch details in parallel (limited)
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut handles = Vec::new();
    for code in codes {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let proxy = proxy_client.clone();
        let headers = headers.clone();
        let wb_min = params.wb_min;
        let h = tokio::spawn(async move {
            let _p = permit;
            let secid = code_to_secid(&code);
            let url = match build_detail_url(&secid) {
                Ok(url) => url,
                Err(_) => return None,
            };
            let v: Value = match proxy_get_json(&proxy, url, &headers).await {
                Ok(v) => v,
                Err(_) => return None,
            };
            let data = v.get("data")?.clone();
            let wb = data.get("f191").and_then(|x| {
                match x {
                    Value::String(s) => normalize_percent_scalar(s.as_str()),
                    Value::Number(n) => n.as_f64(),
                    _ => None,
                }
            });
            if wb.unwrap_or(f64::MIN) < wb_min {
                return None;
            }
            
            // 筛选 f137 > 0
            let f137 = data.get("f137").and_then(|x| x.as_f64());
            // if f137.unwrap_or(0.0) <= 0.0 {
            //     return None;
            // }
            let item = FilteredStockItem {
                f57: data.get("f57").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
                f58: data.get("f58").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
                f43: data.get("f43").and_then(|x| x.as_f64()),
                f170: data.get("f170").and_then(|x| x.as_f64()),
                f50: data.get("f50").and_then(|x| x.as_f64()),
                f168: data.get("f168").and_then(|x| x.as_f64()),
                f191: wb,
                f137,
            };
            Some(item)
        });
        handles.push(h);
    }
    let mut items: Vec<FilteredStockItem> = Vec::new();
    for h in handles {
        if let Ok(Some(item)) = h.await {
            items.push(item);
        }
    }
    if params.limit > 0 && items.len() > params.limit {
        items.truncate(params.limit);
    }
    let out = serde_json::json!({
        "count": items.len(),
        "items": items,
    });
    Ok(out)
}

pub async fn get_filtered_stocks_param_with_proxy(params: FilterParams) -> Result<Value, StockFilterError> {
    let client = Client::new();
    get_filtered_stocks_param(&client, params).await
}

fn em_headers() -> HeaderMap {
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
    headers
}

fn build_list_url(pn: i32, pz: i32) -> Result<Url, StockFilterError> {
    let params = vec![
        ("fs".to_string(), "m:0 t:6,m:0 t:80,m:1 t:2,m:1 t:23".to_string()),
        ("fields".to_string(), "f12,f14,f15,f3,f10,f8".to_string()),
        ("fid".to_string(), "f3".to_string()),
        ("po".to_string(), "1".to_string()),
        ("np".to_string(), "1".to_string()),
        ("fltt".to_string(), "2".to_string()),
        ("invt".to_string(), "2".to_string()),
        ("ut".to_string(), "bd1d9ddb04089700cf9c27f6f7426281".to_string()),
        ("pn".to_string(), pn.to_string()),
        ("pz".to_string(), pz.to_string()),
    ];
    Url::parse_with_params(EM_LIST_URL, params)
        .map_err(|err| StockFilterError::Url(err.to_string()))
}

fn build_detail_url(secid: &str) -> Result<Url, StockFilterError> {
    let params = vec![
        ("secid".to_string(), secid.to_string()),
        ("fields".to_string(), "f57,f58,f43,f170,f50,f168,f191,f137".to_string()),
        ("fltt".to_string(), "2".to_string()),
        ("invt".to_string(), "2".to_string()),
        ("ut".to_string(), "bd1d9ddb04089700cf9c27f6f7426281".to_string()),
    ];
    Url::parse_with_params(EM_DETAIL_URL, params)
        .map_err(|err| StockFilterError::Url(err.to_string()))
}

// proxy_get_json moved to utils::proxy::http for reuse
