use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};

use crate::models::ai_trend_analysis::{AiTrendAnalysis, NewAiTrendAnalysis, UpdateAiTrendAnalysis};
use crate::schema::ai_trend_analysis::dsl::*;

pub type PgPoolConn = PooledConnection<ConnectionManager<PgConnection>>;

/// 创建分析记录
pub fn create(conn: &mut PgPoolConn, new_item: &NewAiTrendAnalysis) -> Result<AiTrendAnalysis, diesel::result::Error> {
    diesel::insert_into(ai_trend_analysis)
        .values(new_item)
        .get_result(conn)
}

/// 根据ID更新分析记录
pub fn update_by_id(
    conn: &mut PgPoolConn,
    record_id: i32,
    update_data: &UpdateAiTrendAnalysis,
) -> Result<AiTrendAnalysis, diesel::result::Error> {
    diesel::update(ai_trend_analysis.filter(id.eq(record_id)))
        .set(update_data)
        .get_result(conn)
}

/// 根据ID查询分析记录
pub fn find_by_id(conn: &mut PgPoolConn, record_id: i32) -> Result<Option<AiTrendAnalysis>, diesel::result::Error> {
    use diesel::OptionalExtension;
    ai_trend_analysis
        .filter(id.eq(record_id))
        .first::<AiTrendAnalysis>(conn)
        .optional()
}

/// 查询历史记录（分页）
pub fn list_history(
    conn: &mut PgPoolConn,
    filter_stock_code: Option<&str>,
    page_size: i64,
    page: i64,
) -> Result<(Vec<AiTrendAnalysis>, i64), diesel::result::Error> {
    let offset = (page - 1) * page_size;

    let mut query = ai_trend_analysis.into_boxed();
    let mut count_query = ai_trend_analysis.into_boxed();

    if let Some(code) = filter_stock_code {
        query = query.filter(stock_code.eq(code));
        count_query = count_query.filter(stock_code.eq(code));
    }

    let total: i64 = count_query.count().get_result(conn)?;

    let results = query
        .order(created_at.desc())
        .limit(page_size)
        .offset(offset)
        .load::<AiTrendAnalysis>(conn)?;

    Ok((results, total))
}
