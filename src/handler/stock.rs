use axum::{extract::Query, http::StatusCode, Json};
use serde_json::Value;
use anyhow::Result;

use crate::routes::stock::{StockQuery, StockResponse, internal_error};
use crate::utils::secid::code_to_secid;

pub async fn get_stock(Query(q): Query<StockQuery>) -> Result<(StatusCode, Json<StockResponse>), (StatusCode, Json<Value>)> {
    if q.source.as_str() != "em" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "unsupported source", "source": q.source})),
        ));
    }

    let secid = code_to_secid(&q.code);
    let url = "https://push2.eastmoney.com/api/qt/stock/get";
    let fields = "f57,f58,f43,f170,f50,f168,f191,f137";

    let client = reqwest::Client::new();
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
    let data = json_body.get("data").cloned().unwrap_or_else(|| serde_json::json!({}));

    let body = StockResponse {
        source: "em".to_string(),
        code: q.code,
        data,
    };
    Ok((StatusCode::OK, Json(body)))
}


