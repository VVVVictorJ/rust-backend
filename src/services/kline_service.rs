use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_ENCODING, REFERER, USER_AGENT};
use reqwest::{Client, Url};
use serde_json::Value;
use std::str::FromStr;
use std::sync::OnceLock;
use thiserror::Error;

use crate::models::NewDailyKline;
use crate::utils::proxy::{proxy_get_json, shared_proxy_client, ProxyError};
use crate::utils::secid::code_to_secid;

const EM_KLINE_URL: &str = "https://push2his.eastmoney.com/api/qt/stock/kline/get";

/// 与 `handler/stock.rs` 中单股行情一致：经代理访问东财时需带浏览器请求头，否则易偶发压缩/非 JSON 网关页导致「decode body」失败。
fn push2his_quote_headers() -> HeaderMap {
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

#[derive(Debug, Error)]
pub enum KlineServiceError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("proxy error: {0}")]
    Proxy(#[from] ProxyError),
    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("no data in response")]
    NoData,
}

#[derive(Debug)]
pub struct KlineParseResult {
    pub stock_code: String,
    pub stock_name: String,
    pub total: usize,
    pub parsed: Vec<NewDailyKline>,
    pub errors: Vec<String>,
}

/// 调用东方财富 K 线接口；`beg`/`end` 对日/月 K 一般为 `YYYYMMDD`。
pub async fn fetch_eastmoney_kline_params(
    _client: &Client,
    stock_code: &str,
    klt: &str,
    beg: &str,
    end: &str,
) -> Result<Value, KlineServiceError> {
    let secid = code_to_secid(stock_code);
    let ut = "fa5fd1943c7b386f172d6893dbfba10b";
    let fields1 = "f1%2Cf2%2Cf3%2Cf4%2Cf5%2Cf6";
    let fields2 = "f51,f52,f53,f54,f55,f56,f57,f58";
    let fqt = "1";
    let smplmt = "460";
    let lmt = "1000000";
    let _timestamp = chrono::Utc::now().timestamp_millis().to_string();

    let url = format!(
        "{EM_KLINE_URL}?secid={secid}&ut={ut}&fields1={fields1}&fields2={fields2}&klt={klt}&fqt={fqt}&beg={beg}&end={end}&smplmt={smplmt}&lmt={lmt}&_={_timestamp}"
    );

    // 与高并发多级筛选共用 `shared_proxy_client`：所有东财 push2his K 线经代理出站。
    let proxy_client = shared_proxy_client()?;
    let headers = push2his_quote_headers();
    let parsed_url =
        Url::parse(&url).map_err(|err| KlineServiceError::ParseError(err.to_string()))?;
    let json = proxy_get_json(&proxy_client, parsed_url, &headers).await?;
    Ok(json)
}

/// 日线 K 线（`klt=101`）。
pub async fn fetch_eastmoney_kline(
    client: &Client,
    stock_code: &str,
    beg_date: &str,
    end_date: &str,
) -> Result<Value, KlineServiceError> {
    fetch_eastmoney_kline_params(client, stock_code, "101", beg_date, end_date).await
}

/// 月 K（`klt=103`）。  
/// **`beg` / `end` 须为东财约定的 `YYYYMMDD`，不是数字下标**。误用 `0`/`2050` 易导致只返回极少数 K 线，月线不足 21 根。
pub async fn fetch_and_parse_monthly_kline_stock(
    client: &Client,
    stock_code: &str,
) -> Result<KlineParseResult, KlineServiceError> {
    const KLT_MONTH: &str = "103";
    const BEG: &str = "19900101";
    const END: &str = "20500101";
    let json_data = fetch_eastmoney_kline_params(client, stock_code, KLT_MONTH, BEG, END).await?;
    parse_kline_json(&json_data)
}

/// 月线拉取：**实际出站走** [`crate::utils::proxy::proxy_get_json`]。`reqwest::Client` 仅为保留旧签名的占位（不再用于直连）。
pub async fn fetch_and_parse_monthly_kline_via_proxy_only(
    stock_code: &str,
) -> Result<KlineParseResult, KlineServiceError> {
    fetch_and_parse_monthly_kline_stock(monthly_placeholder_client_for_monthly_api(), stock_code)
        .await
}

static MONTHLY_KLINE_DUMMY_HTTP: OnceLock<Client> = OnceLock::new();

fn monthly_placeholder_client_for_monthly_api() -> &'static Client {
    MONTHLY_KLINE_DUMMY_HTTP.get_or_init(Client::new)
}

pub fn parse_kline_json(json_data: &Value) -> Result<KlineParseResult, KlineServiceError> {
    let data = json_data.get("data").ok_or(KlineServiceError::NoData)?;

    let stock_code = data
        .get("code")
        .and_then(|v| v.as_str())
        .ok_or_else(|| KlineServiceError::ParseError("Missing stock code".to_string()))?
        .to_string();

    let stock_name = data
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();

    let klines_array = data
        .get("klines")
        .and_then(|v| v.as_array())
        .ok_or_else(|| KlineServiceError::ParseError("No klines array".to_string()))?;

    let total = klines_array.len();
    let mut parsed = Vec::new();
    let mut errors = Vec::new();

    for kline_str in klines_array {
        if let Some(kline) = kline_str.as_str() {
            match parse_single_kline_str(&stock_code, kline) {
                Ok(kline_data) => parsed.push(kline_data),
                Err(e) => errors.push(format!("Parse error for '{kline}': {e}")),
            }
        }
    }

    Ok(KlineParseResult {
        stock_code,
        stock_name,
        total,
        parsed,
        errors,
    })
}

fn parse_single_kline_str(stock_code: &str, kline_str: &str) -> Result<NewDailyKline, String> {
    let parts: Vec<&str> = kline_str.split(',').collect();

    if parts.len() < 7 {
        return Err(format!(
            "Invalid format, expected at least 7 fields, got {}",
            parts.len()
        ));
    }

    let trade_date = NaiveDate::parse_from_str(parts[0], "%Y-%m-%d")
        .or_else(|_| NaiveDate::parse_from_str(parts[0], "%Y%m%d"))
        .map_err(|e| {
            format!(
                "Invalid date '{}' (need YYYY-MM-DD or YYYYMMDD): {}",
                parts[0], e
            )
        })?;

    let open_price = BigDecimal::from_str(parts[1]).unwrap_or_else(|_| BigDecimal::from(0));
    let close_price = BigDecimal::from_str(parts[2]).unwrap_or_else(|_| BigDecimal::from(0));
    let high_price = BigDecimal::from_str(parts[3]).unwrap_or_else(|_| BigDecimal::from(0));
    let low_price = BigDecimal::from_str(parts[4]).unwrap_or_else(|_| BigDecimal::from(0));
    let volume = parts[5].parse::<i64>().unwrap_or(0);
    let amount = BigDecimal::from_str(parts[6]).unwrap_or_else(|_| BigDecimal::from(0));

    Ok(NewDailyKline {
        stock_code: stock_code.to_string(),
        trade_date,
        open_price,
        high_price,
        low_price,
        close_price,
        volume,
        amount,
    })
}

pub async fn fetch_and_parse_kline_data(
    client: &Client,
    stock_code: &str,
    start_date: &str,
    end_date: &str,
) -> Result<KlineParseResult, KlineServiceError> {
    let json_data = fetch_eastmoney_kline(client, stock_code, start_date, end_date).await?;
    parse_kline_json(&json_data)
}
