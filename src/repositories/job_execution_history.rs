use diesel::prelude::*;
use diesel::result::Error as DieselError;

use crate::models::job_execution_history::{JobExecutionHistory, NewJobExecutionHistory, UpdateJobExecutionHistory};
use crate::schema::job_execution_history::dsl::*;

/// 创建任务执行历史记录
pub fn create(
    conn: &mut PgConnection,
    new_history: &NewJobExecutionHistory,
) -> Result<JobExecutionHistory, DieselError> {
    diesel::insert_into(job_execution_history)
        .values(new_history)
        .get_result(conn)
}

/// 根据ID查询
pub fn find_by_id(
    conn: &mut PgConnection,
    history_id: i32,
) -> Result<JobExecutionHistory, DieselError> {
    job_execution_history.find(history_id).first(conn)
}

/// 更新任务执行历史
pub fn update(
    conn: &mut PgConnection,
    history_id: i32,
    update_data: &UpdateJobExecutionHistory,
) -> Result<JobExecutionHistory, DieselError> {
    diesel::update(job_execution_history.find(history_id))
        .set(update_data)
        .get_result(conn)
}

/// 根据任务名查询最新的执行记录
pub fn find_latest_by_job_name(
    conn: &mut PgConnection,
    job_name_filter: &str,
) -> Result<Option<JobExecutionHistory>, DieselError> {
    job_execution_history
        .filter(job_name.eq(job_name_filter))
        .order(started_at.desc())
        .first(conn)
        .optional()
}

/// 分页查询执行历史
pub fn paginate(
    conn: &mut PgConnection,
    job_name_filter: Option<String>,
    status_filter: Option<String>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<JobExecutionHistory>, i64), DieselError> {
    let offset = (page - 1) * page_size;
    
    // 构建基础查询（需要两个独立的查询对象）
    let mut count_query = job_execution_history.into_boxed();
    let mut items_query = job_execution_history.into_boxed();
    
    // 应用筛选条件到两个查询
    if let Some(ref job_name_val) = job_name_filter {
        count_query = count_query.filter(job_name.eq(job_name_val));
        items_query = items_query.filter(job_name.eq(job_name_val));
    }
    
    if let Some(ref status_val) = status_filter {
        count_query = count_query.filter(status.eq(status_val));
        items_query = items_query.filter(status.eq(status_val));
    }
    
    // 获取总数
    let total = count_query.count().get_result(conn)?;
    
    // 获取分页数据
    let items = items_query
        .order(started_at.desc())
        .limit(page_size)
        .offset(offset)
        .load::<JobExecutionHistory>(conn)?;
    
    Ok((items, total))
}

/// 根据任务名查询所有执行记录
pub fn find_by_job_name(
    conn: &mut PgConnection,
    job_name_filter: &str,
    limit: Option<i64>,
) -> Result<Vec<JobExecutionHistory>, DieselError> {
    let mut query = job_execution_history
        .filter(job_name.eq(job_name_filter))
        .order(started_at.desc())
        .into_boxed();
    
    if let Some(limit_val) = limit {
        query = query.limit(limit_val);
    }
    
    query.load(conn)
}

/// 删除指定ID的记录
pub fn delete(
    conn: &mut PgConnection,
    history_id: i32,
) -> Result<usize, DieselError> {
    diesel::delete(job_execution_history.find(history_id)).execute(conn)
}

