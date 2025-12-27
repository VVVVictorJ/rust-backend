use chrono::{DateTime, Utc};
use diesel::prelude::*;
use bigdecimal::BigDecimal;

use crate::schema::profit_analysis;

#[derive(Queryable, Debug, Clone)]
#[diesel(belongs_to(crate::models::stock_snapshots::StockSnapshot, foreign_key = snapshot_id))]
pub struct ProfitAnalysis {
    pub id: i32,
    pub snapshot_id: i32,
    pub strategy_name: String,
    pub profit_rate: BigDecimal,
    pub analysis_time: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = profit_analysis)]
pub struct NewProfitAnalysis {
    pub snapshot_id: i32,
    pub strategy_name: String,
    pub profit_rate: BigDecimal,
}

