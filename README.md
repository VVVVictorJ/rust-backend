# rust-backend（Axum + Diesel + Postgres）

面向“股票信息查询/筛选 + 分析结果落库”的后端服务：

- **对外数据抓取**：从东方财富（EastMoney）接口拉取单只股票行情与批量列表，并用 `polars` 做条件筛选
- **数据持久化**：使用 `Diesel + r2d2` 连接 Postgres，存储请求、请求-股票关联、快照、日K线、收益分析

默认本地地址：`http://127.0.0.1:8001`

## 功能与接口概览

### 基础

- **GET** `/`：`Axum minimal backend`
- **GET** `/healthz`：`ok`

### 股票数据（上游：EastMoney）

- **GET** `/api/stock`
  - query：`code`（6位数字代码，如 `600519`）、`source`（仅支持 `em`，默认 `em`）、`raw_only`（默认 `false`）
- **GET** `/api/stock/filtered/param`
  - query（均有默认值）：
    - `pct_min`/`pct_max`：涨跌幅区间（默认 2~5）
    - `lb_min`：量比下限（默认 5）
    - `hs_min`：换手率下限（默认 1）
    - `wb_min`：委比下限（默认 20）
    - `concurrency`：并发数（默认 8，内部限制 1~64）
    - `limit`：最多返回条数（默认 0=不限制）
    - `pz`：分页大小（默认 1000，内部限制 100~5000）

> 说明：前端里曾出现 `/api/stock/filtered` 的调用，但当前后端仅实现了 `/api/stock/filtered/param`。

### 数据落库（Postgres）

- **POST** `/stock-requests`
- **GET/DELETE** `/stock-requests/:id`
- **POST** `/api/stock-request-stocks`
- **GET/DELETE** `/api/stock-request-stocks/:request_id/:stock_code`
- **POST** `/api/stock-snapshots`
- **GET/DELETE** `/api/stock-snapshots/:id`
- **POST** `/api/profit-analyses`
- **GET/DELETE** `/api/profit-analyses/:id`
- **POST** `/api/daily-klines`
- **GET/DELETE** `/api/daily-klines/:stock_code/:trade_date`

### 股票数据查询（复杂查询）

- **POST** `/api/stock-trade-date-query`
  - 根据交易日期查询股票快照数据（支持分页）
  - 请求体：`{"trade_date": "YYYY-MM-DD", "page": 1, "page_size": 20}`
  - 返回：快照数据列表 + 分页信息（total、total_pages）

- **POST** `/api/stock-price-compare`
  - 查询带盈利等级的股票价格对比数据（支持分页）
  - 请求体：`{"trade_date": "YYYY-MM-DD", "page": 1, "page_size": 20}`
  - 自动计算前一个交易日（考虑周末和节假日）
  - 返回：股票数据 + 盈利等级（A/B/C）+ 分页信息

### 定时任务管理

- **POST** `/api/scheduler/trigger-kline-import`
  - 手动触发 K线导入任务
  - 功能：获取当天入库的股票代码并批量导入K线数据
  - 无需请求体
  - 返回：执行结果统计（total_stocks、success_count、failed_count、details）

- **POST** `/api/scheduler/trigger-profit-analysis`
  - 手动触发盈利分析任务
  - 功能：分析昨日快照与今日K线的盈利情况
  - 无需请求体
  - 返回：执行结果统计（total_snapshots、analyzed_count、details）

> **定时任务说明**：
> - K线导入任务：每天北京时间 **15:01** 自动执行（使用 Asia/Shanghai 时区）
> - 盈利分析任务：每天北京时间 **15:40** 自动执行（使用 Asia/Shanghai 时区）
> - 支持通过 API 手动触发，适用于测试或补录数据

## 技术栈

- **Web**：`axum 0.7`、`tokio`、`tower-http`（Trace/CORS）
- **DB**：`diesel 2.x` + `r2d2`（Postgres）
- **HTTP Client**：`reqwest`（gzip）
- **数据处理**：`polars`（lazy/filter）
- **定时任务**：`tokio-cron-scheduler`、`chrono-tz`（时区支持）
- **配置/日志**：`dotenvy`、`tracing` + `tracing-subscriber`

## 目录结构（关键部分）

- `src/app.rs`：创建 Postgres 连接池、注入 `AppState`、挂载中间件（CORS/Trace）
- `src/routes/`：路由定义（/api 与 /stock-requests）
- `src/handler/`：HTTP handler（请求解析、调用 repo/service、统一错误）
- `src/repositories/`：Diesel 查询/插入/删除
- `src/models/` + `src/schema.rs`：数据库模型与 Diesel schema
- `src/services/stock_filter.rs`：批量股票抓取 + polars 条件筛选
- `src/scheduler/`：定时任务定义（K线导入、盈利分析）
  - `kline_import_job.rs`：每天 15:01 自动导入K线数据
  - `profit_analysis_job.rs`：每天 15:40 自动执行盈利分析
- `src/asset/DDL/stock_create.sql`：数据库建表 SQL（当前仓库未提供 `migrations/` 目录）
- `src/asset/test/api_examples.txt`：更多 curl 示例
- `src/asset/test/api_test_guide.md`：完整的 API 测试指南

## 配置（环境变量）

项目启动时会尝试加载 `.env`（`dotenvy`）。

- **必需**
  - `DATABASE_URL`：Postgres 连接串（未设置会直接 panic）
- **可选**
  - `HOST`：监听地址，默认 `127.0.0.1`
  - `PORT`：监听端口，默认 `8001`
  - `ALLOWED_ORIGINS`：CORS 白名单（逗号分隔）。未设置时默认允许：
    - `http://localhost:5173`
    - `http://127.0.0.1:5173`
  - `RUST_LOG`：日志级别（默认 `info,tower_http=info,axum=info`）

`.env` 示例：

```dotenv
DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/stock
HOST=127.0.0.1
PORT=8001
ALLOWED_ORIGINS=http://localhost:5173,http://127.0.0.1:5173
RUST_LOG=info,tower_http=info,axum=info
```

## 数据库准备与初始化

### 1) 准备 Postgres

该项目的建表 SQL 使用了 `gen_random_uuid()`，通常需要启用扩展 `pgcrypto`：

```sql
CREATE EXTENSION IF NOT EXISTS pgcrypto;
```

### 2) 执行建表脚本

脚本位置：`src/asset/DDL/stock_create.sql`

使用 `psql` 示例：

```bash
psql "$DATABASE_URL" -c "CREATE EXTENSION IF NOT EXISTS pgcrypto;"
psql "$DATABASE_URL" -f rust-backend/src/asset/DDL/stock_create.sql
```

> 备注：仓库里有 `diesel.toml`，但当前未提供 `migrations/` 目录；如后续补齐 migrations，可再切换为 Diesel CLI 管理迁移。

## 本地运行（Windows / PowerShell）

```powershell
cd rust-backend
$env:DATABASE_URL='postgres://postgres:postgres@127.0.0.1:5432/stock'
$env:HOST='127.0.0.1'
$env:PORT='8001'
$env:RUST_LOG='info,tower_http=info,axum=info'
cargo run
```

启动后访问：`http://127.0.0.1:8001/healthz`

## Diesel 迁移

```
diesel migration
```

## Docker 运行

### 仅运行后端容器（需要外部 Postgres）

```bash
docker build -t rust-backend:local ./rust-backend
docker run --rm -p 8000:8000 ^
  -e HOST=0.0.0.0 ^
  -e PORT=8000 ^
  -e DATABASE_URL="postgres://user:pass@host.docker.internal:5432/stock" ^
  rust-backend:local
```

> 提示：仓库根目录下有 `docker/docker-compose.axum.yml`（前端+后端）。使用 compose 时同样需要为后端服务补齐 `DATABASE_URL`（可通过 `environment` 或 `env_file`）。

## API 调用示例（curl）

### 1) 查询单只股票（EastMoney）

```bash
curl "http://localhost:8001/api/stock?code=600519&source=em&raw_only=false"
```

### 2) 条件筛选（EastMoney + polars）

```bash
curl "http://localhost:8001/api/stock/filtered/param?pct_min=2&pct_max=5&lb_min=5&hs_min=1&wb_min=20&concurrency=8&limit=30&pz=1000"
```

### 3) 创建/查询/删除 stock_request

```bash
curl -i -X POST http://localhost:8001/stock-requests \
  -H "Content-Type: application/json" \
  -d '{"strategy_name":"demo","time_range_start":"2024-01-01"}'

curl -i http://localhost:8001/stock-requests/1
curl -i -X DELETE http://localhost:8001/stock-requests/1
```

### 4) 交易日查询

```bash
# 查询 2025-12-30 的股票快照数据（第1页，每页20条）
curl -X POST http://localhost:8001/api/stock-trade-date-query \
  -H "Content-Type: application/json" \
  -d '{"trade_date":"2025-12-30","page":1,"page_size":20}'
```

### 5) 价格对比查询（带盈利等级）

```bash
# 查询 2025-12-30 的价格对比数据
# 自动查询前一个交易日的快照数据并对比
curl -X POST http://localhost:8001/api/stock-price-compare \
  -H "Content-Type: application/json" \
  -d '{"trade_date":"2025-12-30","page":1,"page_size":20}'
```

### 6) 手动触发定时任务

```bash
# 手动触发 K线导入任务
curl -X POST http://localhost:8001/api/scheduler/trigger-kline-import

# 手动触发盈利分析任务
curl -X POST http://localhost:8001/api/scheduler/trigger-profit-analysis
```

### 7) 其他 CRUD 示例

更完整的示例见：`src/asset/test/api_examples.txt` 和 `src/asset/test/api_test_guide.md`

## 错误码约定（后端返回）

- `404 Not Found`：资源不存在（如 `GET/DELETE` 某个 id/pk）
- `400 Bad Request`：部分插入场景会把数据库约束/外键/唯一键错误映射为 400（如 `daily_klines`、`stock_request_stocks`）
- `500 Internal Server Error`：连接池/未知 DB 错误等
- `502 Bad Gateway`：上游 EastMoney 返回非 2xx（仅股票抓取相关接口）

## 常见问题（Windows）

### 1) 构建时报 “拒绝访问 (os error 5)”

通常是 `rust-backend.exe` 仍在运行或被占用：

```powershell
Get-Process rust-backend -ErrorAction SilentlyContinue | Stop-Process -Force
```

### 2) 端口被占用（默认 8001）

```powershell
netstat -ano | findstr :8001
taskkill /PID <PID> /F
```
