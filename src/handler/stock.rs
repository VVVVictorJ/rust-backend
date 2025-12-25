use anyhow::Result;
use axum::{extract::Query, http::StatusCode, Json};
use serde_json::Value;

use crate::routes::stock::{internal_error, StockQuery};
use crate::utils::secid::code_to_secid;
 use crate::services::stock_filter::{FilterParams, get_filtered_stocks_param as svc_get_filtered_stocks_param};

pub async fn get_stock(
    Query(q): Query<StockQuery>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    if q.source.as_str() != "em" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "unsupported source", "source": q.source})),
        ));
    }

    let secid = code_to_secid(&q.code);
    let url = "https://push2.eastmoney.com/api/qt/stock/get";
    let fields = "f57,f58,f43,f170,f50,f168,f191,f137";

     let client = {
         use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT, REFERER, ACCEPT_ENCODING};
         let mut headers = HeaderMap::new();
         headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));
         headers.insert(ACCEPT, HeaderValue::from_static("application/json, text/plain, */*"));
         headers.insert(REFERER, HeaderValue::from_static("https://quote.eastmoney.com"));
         headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip"));
         reqwest::Client::builder()
             .default_headers(headers)
             .build()
             .unwrap()
     };
    let resp = client
        .get(url)
        .query(&[
            ("secid", secid.as_str()),
            ("fields", fields),
            ("fltt", "2"),
            ("invt", "2"),
            ("ut", "bd1d9ddb04089700cf9c27f6f7426281"),
        ])
        .send()
        .await
        .map_err(internal_error)?;

    let status = resp.status();
    if !status.is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"error": "upstream error", "status": status.as_u16()})),
        ));
    }

    let json_body: Value = resp.json().await.map_err(internal_error)?;
    let response_body = if q.raw_only {
        json_body
    } else {
        let data = json_body
            .get("data")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        serde_json::json!({
            "source": "em",
            "code": q.code,
            "data": data,
        })
    };

    Ok((StatusCode::OK, Json(response_body)))
}

 #[derive(Debug, serde::Deserialize)]
 pub struct FilterParamQuery {
     #[serde(default = "default_pct_min")]
     pub pct_min: f64,
     #[serde(default = "default_pct_max")]
     pub pct_max: f64,
     #[serde(default = "default_lb_min")]
     pub lb_min: f64,
     #[serde(default = "default_hs_min")]
     pub hs_min: f64,
     #[serde(default = "default_wb_min")]
     pub wb_min: f64,
     #[serde(default = "default_concurrency")]
     pub concurrency: i32,
     #[serde(default)]
     pub limit: i32,
     #[serde(default = "default_pz")]
     pub pz: i32,
 }
 
 fn default_pct_min() -> f64 { 2.0 }
 fn default_pct_max() -> f64 { 5.0 }
 fn default_lb_min() -> f64 { 5.0 }
 fn default_hs_min() -> f64 { 1.0 }
 fn default_wb_min() -> f64 { 20.0 }
 fn default_concurrency() -> i32 { 8 }
 fn default_pz() -> i32 { 1000 }
 
 pub async fn get_filtered_stocks_param(
     Query(p): Query<FilterParamQuery>,
 ) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
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
     let client = {
         use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT, REFERER, ACCEPT_ENCODING};
         let mut headers = HeaderMap::new();
         headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));
         headers.insert(ACCEPT, HeaderValue::from_static("application/json, text/plain, */*"));
         headers.insert(REFERER, HeaderValue::from_static("https://quote.eastmoney.com"));
         headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip"));
         reqwest::Client::builder()
             .default_headers(headers)
             .build()
             .unwrap()
     };
     match svc_get_filtered_stocks_param(&client, params).await {
         Ok(v) => Ok((StatusCode::OK, Json(v))),
         Err(e) => Err(internal_error(e)),
     }
 }
 