use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::schema::job_execution_history;

#[derive(Queryable, Selectable, Serialize, Clone, Debug)]
#[diesel(table_name = job_execution_history)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct JobExecutionHistory {
    pub id: i32,
    pub job_name: String,
    pub status: String,
    pub started_at: NaiveDateTime,
    pub completed_at: Option<NaiveDateTime>,
    pub total_count: i32,
    pub success_count: i32,
    pub failed_count: i32,
    pub skipped_count: i32,
    pub details: Option<Value>,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = job_execution_history)]
pub struct NewJobExecutionHistory {
    pub job_name: String,
    pub status: String,
    pub started_at: NaiveDateTime,
    pub completed_at: Option<NaiveDateTime>,
    pub total_count: i32,
    pub success_count: i32,
    pub failed_count: i32,
    pub skipped_count: i32,
    pub details: Option<Value>,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = job_execution_history)]
pub struct UpdateJobExecutionHistory {
    pub status: Option<String>,
    pub completed_at: Option<NaiveDateTime>,
    pub total_count: Option<i32>,
    pub success_count: Option<i32>,
    pub failed_count: Option<i32>,
    pub skipped_count: Option<i32>,
    pub details: Option<Value>,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
}

