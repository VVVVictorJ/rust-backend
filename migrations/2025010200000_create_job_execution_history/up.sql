-- 创建任务执行历史表
CREATE TABLE job_execution_history (
    id SERIAL PRIMARY KEY,
    job_name VARCHAR(100) NOT NULL,
    status VARCHAR(20) NOT NULL,  -- running, success, failed, partial
    started_at TIMESTAMP NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP,
    total_count INTEGER DEFAULT 0,
    success_count INTEGER DEFAULT 0,
    failed_count INTEGER DEFAULT 0,
    skipped_count INTEGER DEFAULT 0,
    details JSONB,
    error_message TEXT,
    duration_ms BIGINT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- 创建索引
CREATE INDEX idx_job_execution_job_name ON job_execution_history(job_name);
CREATE INDEX idx_job_execution_started_at ON job_execution_history(started_at DESC);

-- 添加注释
COMMENT ON TABLE job_execution_history IS '定时任务执行历史记录表';
COMMENT ON COLUMN job_execution_history.job_name IS '任务名称';
COMMENT ON COLUMN job_execution_history.status IS '执行状态: running/success/failed/partial';
COMMENT ON COLUMN job_execution_history.started_at IS '任务开始时间';
COMMENT ON COLUMN job_execution_history.completed_at IS '任务完成时间';
COMMENT ON COLUMN job_execution_history.total_count IS '总处理数量';
COMMENT ON COLUMN job_execution_history.success_count IS '成功数量';
COMMENT ON COLUMN job_execution_history.failed_count IS '失败数量';
COMMENT ON COLUMN job_execution_history.skipped_count IS '跳过数量';
COMMENT ON COLUMN job_execution_history.details IS '详细信息(JSON格式)';
COMMENT ON COLUMN job_execution_history.error_message IS '错误信息';
COMMENT ON COLUMN job_execution_history.duration_ms IS '执行耗时(毫秒)';

