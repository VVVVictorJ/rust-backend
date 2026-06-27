use reqwest::Client;
use serde::Deserialize;

use crate::api_models::bagua::AlmanacResponse;
use crate::handler::error::AppError;

#[derive(Debug, Deserialize)]
struct TiaxAlmanacResponse {
    #[serde(rename = "干支日期")]
    ganzhi_date: String,
}

pub async fn fetch_almanac(year: &str, month: &str, day: &str) -> Result<AlmanacResponse, AppError> {
    let client = Client::new();
    let resp = client
        .get("https://api.tiax.cn/almanac/")
        .query(&[("year", year), ("month", month), ("day", day)])
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch almanac: {}", e);
            AppError::InternalServerError
        })?;

    if !resp.status().is_success() {
        tracing::error!("Almanac API returned status: {}", resp.status());
        return Err(AppError::InternalServerError);
    }

    let body: TiaxAlmanacResponse = resp.json().await.map_err(|e| {
        tracing::error!("Failed to parse almanac response: {}", e);
        AppError::InternalServerError
    })?;

    let ganzhi_date = body.ganzhi_date.trim().to_string();
    let (year_stem, year_branch) = parse_year_ganzhi(&ganzhi_date)?;

    Ok(AlmanacResponse {
        year_stem,
        year_branch,
        ganzhi_date,
    })
}

fn parse_year_ganzhi(ganzhi_date: &str) -> Result<(String, String), AppError> {
    let chars: Vec<char> = ganzhi_date.chars().collect();
    if chars.len() < 2 {
        return Err(AppError::BadRequest(
            "干支日期格式无效".to_string(),
        ));
    }

    Ok((chars[0].to_string(), chars[1].to_string()))
}

#[cfg(test)]
mod tests {
    use super::parse_year_ganzhi;

    #[test]
    fn parse_2026_06_27_ganzhi_date() {
        let (stem, branch) = parse_year_ganzhi("丙午年 甲午月 壬申日").unwrap();
        assert_eq!(stem, "丙");
        assert_eq!(branch, "午");
    }
}
