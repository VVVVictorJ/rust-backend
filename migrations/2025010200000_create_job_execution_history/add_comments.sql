-- 为已创建的表添加注释（UTF8编码）

-- 表注释
COMMENT ON TABLE job_execution_history IS 'Task execution history records';

-- 字段注释
COMMENT ON COLUMN job_execution_history.id IS 'Primary key';
COMMENT ON COLUMN job_execution_history.job_name IS 'Task name: kline_import or profit_analysis';
COMMENT ON COLUMN job_execution_history.status IS 'Execution status: running, success, failed, partial';
COMMENT ON COLUMN job_execution_history.started_at IS 'Start time';
COMMENT ON COLUMN job_execution_history.completed_at IS 'Completion time';
COMMENT ON COLUMN job_execution_history.total_count IS 'Total count';
COMMENT ON COLUMN job_execution_history.success_count IS 'Success count';
COMMENT ON COLUMN job_execution_history.failed_count IS 'Failed count';
COMMENT ON COLUMN job_execution_history.skipped_count IS 'Skipped count';
COMMENT ON COLUMN job_execution_history.details IS 'Detailed execution info (JSON)';
COMMENT ON COLUMN job_execution_history.error_message IS 'Error message';
COMMENT ON COLUMN job_execution_history.duration_ms IS 'Execution duration (ms)';
COMMENT ON COLUMN job_execution_history.created_at IS 'Record creation time';

