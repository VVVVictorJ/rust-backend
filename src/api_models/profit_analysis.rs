use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

#[derive(Debug, Serialize)]
pub struct ProfitAnalysisResponse {
    pub id: i32,
    pub snapshot_id: i32,
    pub strategy_name: String,
    pub profit_rate: BigDecimal,
    pub analysis_time: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProfitAnalysis {
    pub snapshot_id: i32,
    pub strategy_name: String,
    pub profit_rate: BigDecimal,
}

