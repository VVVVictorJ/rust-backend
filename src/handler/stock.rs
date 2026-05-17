use axum::{extract::Query, http::StatusCode, Json};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_ENCODING, REFERER, USER_AGENT};
use reqwest::Url;
use serde_json::Value;

use crate::routes::stock::{internal_error, StockQuery};
use crate::utils::http_client::create_em_client;
use crate::utils::proxy::{proxy_get_json, shared_proxy_client};
use crate::utils::secid::code_to_secid;

fn build_em_stock_get_url(secid: &str, fields: &str) -> Url {
    let mut url =
        Url::parse("https://push2.eastmoney.com/api/qt/stock/get").expect("valid static URL");
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("secid", secid);
        pairs.append_pair("fields", fields);
        pairs.append_pair("fltt", "2");
        pairs.append_pair("invt", "2");
        pairs.append_pair("ut", "bd1d9ddb04089700cf9c27f6f7426281");
    }
    url
}

fn em_quote_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        ),
    );
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/json, text/plain, */*"),
    );
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://quote.eastmoney.com"),
    );
    headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip"));
    headers
}

async fn fetch_em_stock_json_direct(
    url: &Url,
    headers: &HeaderMap,
) -> Result<Value, (StatusCode, Json<Value>)> {
    let client = create_em_client().map_err(internal_error)?;
    let resp = client
        .get(url.clone())
        .headers(headers.clone())
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
    resp.json().await.map_err(internal_error)
}

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
    let fields = "f57,f58,f43,f170,f50,f168,f191,f137";
    let url = build_em_stock_get_url(secid.as_str(), fields);
    let headers = em_quote_headers();

    // 云上主机常无法直连东方财富：与筛选/K 线一致，优先走 PROXY_* 代理，失败再直连（本地开发）
    let json_body: Value = match shared_proxy_client() {
        Ok(proxy_arc) => match proxy_get_json(&proxy_arc, url.clone(), &headers).await {
            Ok(v) => v,
            Err(err) => {
                tracing::warn!(target: "stock", "eastmoney push2 quote via proxy failed: {err}");
                fetch_em_stock_json_direct(&url, &headers).await?
            }
        },
        Err(err) => {
            tracing::debug!(target: "stock", "proxy client unavailable (single quote): {err}");
            fetch_em_stock_json_direct(&url, &headers).await?
        }
    };

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

fn default_pct_min() -> f64 {
    2.0
}
fn default_pct_max() -> f64 {
    5.0
}
fn default_lb_min() -> f64 {
    5.0
}
fn default_hs_min() -> f64 {
    1.0
}
fn default_wb_min() -> f64 {
    20.0
}
fn default_concurrency() -> i32 {
    8
}
fn default_pz() -> i32 {
    1000
}
