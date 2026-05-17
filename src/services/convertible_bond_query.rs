use anyhow::{anyhow, Result};
use chrono::{Datelike, Local, NaiveDate, NaiveDateTime};
use serde_json::Value;

use crate::api_models::convertible_bond_query::ConvertibleBondItem;
use crate::utils::http_client::create_em_client;

const CONVERTIBLE_BOND_URL: &str = "https://datacenter-web.eastmoney.com/api/data/v1/get?sortColumns=PUBLIC_START_DATE,SECURITY_CODE&sortTypes=-1,-1&pageSize=500&pageNumber=1&reportName=RPT_BOND_CB_LIST&columns=ALL&quoteColumns=f2~01~CONVERT_STOCK_CODE~CONVERT_STOCK_PRICE,f235~10~SECURITY_CODE~TRANSFER_PRICE,f236~10~SECURITY_CODE~TRANSFER_VALUE,f2~10~SECURITY_CODE~CURRENT_BOND_PRICE,f237~10~SECURITY_CODE~TRANSFER_PREMIUM_RATIO,f239~10~SECURITY_CODE~RESALE_TRIG_PRICE,f240~10~SECURITY_CODE~REDEEM_TRIG_PRICE,f23~01~CONVERT_STOCK_CODE~PBV_RATIO&quoteType=0&source=WEB&client=WEB";

pub async fn fetch_filtered_convertible_bonds() -> Result<Vec<ConvertibleBondItem>> {
    let client = create_em_client().map_err(|e| anyhow!("create em http client: {e}"))?;
    let text = client
        .get(CONVERTIBLE_BOND_URL)
        .send()
        .await?
        .text()
        .await?;

    let payload = extract_json_payload(&text).ok_or_else(|| anyhow!("invalid jsonp payload"))?;
    let root: Value = serde_json::from_str(payload)?;
    let rows = root
        .pointer("/result/data")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("result.data is missing"))?;

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

    Ok(items)
}

fn extract_json_payload(raw: &str) -> Option<&str> {
    let trimmed = raw.trim();
    if trimmed.starts_with('{') {
        return Some(trimmed);
    }

    let left = trimmed.find('(')?;
    let right = trimmed.rfind(')')?;
    if right <= left {
        return None;
    }
    Some(trimmed[left + 1..right].trim())
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
