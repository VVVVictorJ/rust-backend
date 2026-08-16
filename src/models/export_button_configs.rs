use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde_json::Value;

use crate::schema::export_button_config;

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = export_button_config)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ExportButtonConfig {
    pub id: i32,
    pub page_key: String,
    pub name: String,
    pub plate_codes: Value,
    pub sort_order: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = export_button_config)]
pub struct NewExportButtonConfig {
    pub page_key: String,
    pub name: String,
    pub plate_codes: Value,
    pub sort_order: Option<i32>,
}

#[derive(AsChangeset, Debug, Default, Clone)]
#[diesel(table_name = export_button_config)]
pub struct UpdateExportButtonConfig {
    pub page_key: Option<String>,
    pub name: Option<String>,
    pub plate_codes: Option<Value>,
    pub sort_order: Option<i32>,
    pub updated_at: Option<NaiveDateTime>,
}
