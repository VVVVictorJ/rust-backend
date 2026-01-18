use reqwest::Client;
use serde_json::Value;
use thiserror::Error;

use crate::api_models::stock_plate_em::{EmPlateItem, EmPlateResponse};
use crate::utils::secid::code_to_secid;

const EM_PLATE_URL: &str = "https://push2.eastmoney.com/api/qt/slist/get";

#[derive(Debug, Error)]
pub enum EmPlateError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("missing data field")]
    MissingData,
}

pub async fn fetch_em_plate_list(
    client: &Client,
    stock_code: &str,
) -> Result<EmPlateResponse, EmPlateError> {
    let ut = "fa5fd1943c7b386f172d6893dbfba10b";
    let fields = "f14,f12";
    let secid = code_to_secid(stock_code);
    let fltt = "1";
    let invt = "2";
    let pi = "0";
    let po = "1";
    let np = "1";
    let pz = "500";
    let spt = "3";
    let wbp2u = "|0|0|0|web";
    let timestamp = chrono::Utc::now().timestamp_millis().to_string();

    let resp = client
        .get(EM_PLATE_URL)
        .query(&[
            ("fltt", fltt),
            ("invt", invt),
            ("fields", fields),
            ("secid", secid.as_str()),
            ("ut", ut),
            ("pi", pi),
            ("po", po),
            ("np", np),
            ("pz", pz),
            ("spt", spt),
            ("wbp2u", wbp2u),
            ("_", timestamp.as_str()),
        ])
        .send()
        .await?;

    let body = resp.text().await?;
    let json: Value = serde_json::from_str(&body)?;
    let data = json.get("data").ok_or(EmPlateError::MissingData)?;
    let total = data.get("total").and_then(|v| v.as_i64()).unwrap_or(0);
    let mut items = Vec::new();
    if let Some(diff) = data.get("diff").and_then(|v| v.as_array()) {
        for item in diff {
            let plate_code = item
                .get("f12")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let name = item
                .get("f14")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            if !plate_code.is_empty() && !name.is_empty() {
                items.push(EmPlateItem { plate_code, name });
            }
        }
    }

    Ok(EmPlateResponse { total, items })
}
