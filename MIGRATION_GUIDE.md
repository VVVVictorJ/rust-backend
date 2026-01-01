# 数据库迁移指南

## 定时任务管理功能 - 数据库迁移

### 迁移说明

本次更新添加了定时任务管理功能，需要创建 `job_execution_history` 表来记录任务执行历史。

### 方法1：使用 Diesel CLI（推荐）

如果已安装 `diesel_cli`，可以直接运行：

```bash
cd rust-backend
diesel migration run --database-url "postgres://stock_user:stock_pass@localhost:5432/stock_db"
```

### 方法2：手动执行 SQL

如果没有安装 `diesel_cli`，可以手动执行迁移 SQL：

#### Windows (PowerShell)

```powershell
$env:PGPASSWORD="stock_pass"
psql -h localhost -U stock_user -d stock_db -f "rust-backend\migrations\2025010200000_create_job_execution_history\up.sql"
```

#### Linux/Mac

```bash
export PGPASSWORD="stock_pass"
psql -h localhost -U stock_user -d stock_db -f "rust-backend/migrations/2025010200000_create_job_execution_history/up.sql"
```

### 方法3：使用 Docker 容器内的 psql

如果数据库运行在 Docker 容器中：

```bash
docker exec -i postgres_container_name psql -U stock_user -d stock_db < rust-backend/migrations/2025010200000_create_job_execution_history/up.sql
```

### 验证迁移

迁移成功后，可以验证表是否创建：

```sql
\c stock_db
\dt job_execution_history
SELECT * FROM job_execution_history;
```

### 迁移内容

迁移将创建以下表：

- **表名**: `job_execution_history`
- **字段**:
  - `id`: 主键，自增
  - `job_name`: 任务名称（如 `kline_import`, `profit_analysis`）
  - `status`: 执行状态（`running`, `success`, `failed`, `partial`）
  - `started_at`: 开始时间
  - `completed_at`: 完成时间（可选）
  - `total_count`: 总数量
  - `success_count`: 成功数量
  - `failed_count`: 失败数量
  - `skipped_count`: 跳过数量
  - `details`: JSON 格式的详细信息（可选）
  - `error_message`: 错误信息（可选）
  - `duration_ms`: 执行时长（毫秒）
  - `created_at`: 记录创建时间（默认当前时间）

### 回滚迁移

如果需要回滚迁移：

#### 使用 Diesel CLI

```bash
cd rust-backend
diesel migration revert --database-url "postgres://stock_user:stock_pass@localhost:5432/stock_db"
```

#### 手动执行

```bash
# Windows PowerShell
$env:PGPASSWORD="stock_pass"
psql -h localhost -U stock_user -d stock_db -f "rust-backend\migrations\2025010200000_create_job_execution_history\down.sql"

# Linux/Mac
export PGPASSWORD="stock_pass"
psql -h localhost -U stock_user -d stock_db -f "rust-backend/migrations/2025010200000_create_job_execution_history/down.sql"
```

### 注意事项

1. 请确保数据库用户有足够的权限创建表
2. 如果使用的数据库连接信息与示例不同，请相应修改命令中的参数
3. 建议在执行迁移前备份数据库
4. 迁移执行成功后，需要重启 Rust 后端服务以使用新功能

