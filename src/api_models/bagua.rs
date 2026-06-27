use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct HetuLookupCell {
    pub row_key: String,
    pub col_key: String,
    pub value: i16,
}

#[derive(Debug, Serialize)]
pub struct HetuLookupResponse {
    pub matrix_code: String,
    pub cells: Vec<HetuLookupCell>,
}

#[derive(Debug, Deserialize)]
pub struct HetuLookupQuery {
    pub row: Option<String>,
    pub col: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HetuSingleLookupResponse {
    pub row_key: String,
    pub col_key: String,
    pub value: i16,
}

#[derive(Debug, Deserialize)]
pub struct AlmanacQuery {
    pub year: String,
    pub month: String,
    pub day: String,
}

#[derive(Debug, Serialize)]
pub struct AlmanacResponse {
    pub ganzhi_date: String,
    pub year_stem: String,
    pub year_branch: String,
}
