use anyhow::Result;
use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use reqwest::Client;
use serde_json::Value;
use std::str::FromStr;
use thiserror::Error;

use crate::models::NewDailyKline;
use crate::utils::secid::code_to_secid;

const EM_KLINE_URL: &str = "https://push2his.eastmoney.com/api/qt/stock/kline/get";

#[derive(Debug, Error)]
pub enum KlineServiceError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
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

pub async fn fetch_eastmoney_kline(
    client: &Client,
    stock_code: &str,
    beg_date: &str,
    end_date: &str,
) -> Result<Value, KlineServiceError> {
    let secid = code_to_secid(stock_code);
    let ut = "fa5fd1943c7b386f172d6893dbfba10b";
    let fields1 = "f1%2Cf2%2Cf3%2Cf4%2Cf5%2Cf6";
    let fields2 = "f51,f52,f53,f54,f55,f56,f57,f58";
    let klt = "101";
    let fqt = "1";
    let smplmt = "460";
    let lmt = "1000000";
    let _timestamp = chrono::Utc::now().timestamp_millis().to_string();

    let url = format!(
        "{}?secid={}&ut={}&fields1={}&fields2={}&klt={}&fqt={}&beg={}&end={}&smplmt={}&lmt={}&_={}",
        EM_KLINE_URL, secid, ut, fields1, fields2, klt, fqt, beg_date, end_date, smplmt, lmt, _timestamp
    );

    let resp = client.get(&url).send().await?;
    let body = resp.text().await?;
    let json: Value = serde_json::from_str(&body)?;
    Ok(json)
}

pub fn parse_kline_json(json_data: &Value) -> Result<KlineParseResult, KlineServiceError> {
    let data = json_data
        .get("data")
        .ok_or(KlineServiceError::NoData)?;

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
                Err(e) => errors.push(format!("Parse error for '{}': {}", kline, e)),
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
        return Err(format!("Invalid format, expected at least 7 fields, got {}", parts.len()));
    }

    let trade_date = NaiveDate::parse_from_str(parts[0], "%Y-%m-%d")
        .map_err(|e| format!("Invalid date '{}': {}", parts[0], e))?;

    let open_price = BigDecimal::from_str(parts[1])
        .unwrap_or_else(|_| BigDecimal::from(0));
    let close_price = BigDecimal::from_str(parts[2])
        .unwrap_or_else(|_| BigDecimal::from(0));
    let high_price = BigDecimal::from_str(parts[3])
        .unwrap_or_else(|_| BigDecimal::from(0));
    let low_price = BigDecimal::from_str(parts[4])
        .unwrap_or_else(|_| BigDecimal::from(0));
    let volume = parts[5].parse::<i64>()
        .unwrap_or(0);
    let amount = BigDecimal::from_str(parts[6])
        .unwrap_or_else(|_| BigDecimal::from(0));

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

