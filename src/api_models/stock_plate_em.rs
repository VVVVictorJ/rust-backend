use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct EmPlateItem {
    pub plate_code: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct EmPlateResponse {
    pub total: i64,
    pub items: Vec<EmPlateItem>,
}
