use diesel::prelude::*;

#[derive(Queryable, Debug, Clone)]
#[diesel(table_name = crate::schema::he_luo_lookup)]
#[allow(dead_code)]
pub struct HeLuoLookup {
    pub id: i32,
    pub matrix_code: String,
    pub row_key: String,
    pub col_key: String,
    pub value: i16,
}
