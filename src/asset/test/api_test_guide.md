# API 测试指南

**基础地址**: `http://localhost:8001`
**API 前缀**: `/api`
**Content-Type**: `application/json`

---

## 目录

- [1. 基础接口](#1-基础接口)
- [2. 股票数据查询接口（EastMoney）](#2-股票数据查询接口eastmoney)
- [3. 股票请求管理](#3-股票请求管理)
- [4. 股票请求关联管理](#4-股票请求关联管理)
- [5. 股票快照管理](#5-股票快照管理)
- [6. 日K线数据管理](#6-日k线数据管理)
- [7. 盈利分析管理](#7-盈利分析管理)
- [8. 定时任务管理](#8-定时任务管理)

---

## 1. 基础接口

### 1.1 根路径

```bash
curl -i http://localhost:8001/
```

**预期返回**: `200 OK` + `"Axum minimal backend"`

### 1.2 健康检查

```bash
curl -i http://localhost:8001/healthz
```

**预期返回**: `200 OK` + `"ok"`

---

## 2. 股票数据查询接口（EastMoney）

### 2.1 查询单只股票

**接口**: `GET /api/stock`

**参数**:

- `code`: 股票代码（6位数字，如 `600519`）
- `source`: 数据源（默认 `em`，目前仅支持东方财富）
- `raw_only`: 是否只返回原始数据（默认 `false`）

**示例**:

```bash
# 查询贵州茅台（600519）
curl -i "http://localhost:8001/api/stock?code=600519&source=em&raw_only=false"
```

**预期返回**: `200 OK` + JSON 数据

```json
{
  "source": "em",
  "code": "600519",
  "data": {
    "f57": "600519",
    "f58": "贵州茅台",
    "f43": 1800.50,
    ...
  }
}
```

---

### 2.2 条件筛选股票（带数据落库）

**接口**: `GET /api/stock/filtered/param`

**参数**（均有默认值）:

- `pct_min`: 涨跌幅下限（默认 `2.0`）
- `pct_max`: 涨跌幅上限（默认 `5.0`）
- `lb_min`: 量比下限（默认 `5.0`）
- `hs_min`: 换手率下限（默认 `1.0`）
- `wb_min`: 委比下限（默认 `20.0`）
- `concurrency`: 并发数（默认 `8`，限制 1~64）
- `limit`: 最多返回条数（默认 `0` = 不限制）
- `pz`: 分页大小（默认 `1000`，限制 100~5000）

**特性**:

- 返回非空结果时，自动将请求记录到 `stock_requests` 表
- 自动将股票快照数据记录到 `stock_snapshots` 表

**示例**:

```bash
# 使用默认参数
curl -i "http://localhost:8001/api/stock/filtered/param"

# 自定义参数
curl -i "http://localhost:8001/api/stock/filtered/param?pct_min=2&pct_max=5&lb_min=5&hs_min=1&wb_min=20&concurrency=8&limit=30&pz=1000"
```

**预期返回**: `200 OK` + JSON 数据

```json
{
  "count": 4,
  "items": [
    {
      "f57": "301079",
      "f58": "邵阳液压",
      "f43": 31.87,
      "f170": 4.59,
      "f50": 2.08,
      "f168": 43.08,
      "f191": 49.17,
      "f137": 24656181.0
    },
    ...
  ]
}
```

**字段说明**:

- `f57`: 股票代码
- `f58`: 股票名称
- `f43`: 最新价
- `f170`: 涨跌幅（%）
- `f50`: 量比
- `f168`: 换手率（%）
- `f191`: 委比
- `f137`: 主力资金净流入（元）

---

## 3. 股票请求管理

### 3.1 创建股票请求

**接口**: `POST /stock-requests`

**请求体**:

```json
{
  "strategy_name": "momentum_v1",
  "time_range_start": "2024-01-01",
  "time_range_end": "2024-12-31"
}
```

**示例**:

```bash
curl -i -X POST http://localhost:8001/stock-requests \
  -H "Content-Type: application/json" \
  -d '{
    "strategy_name": "momentum_v1",
    "time_range_start": "2024-01-01",
    "time_range_end": "2024-12-31"
  }'
```

**预期返回**: `200 OK` + JSON

```json
{
  "id": 1,
  "request_uuid": "550e8400-e29b-41d4-a716-446655440000",
  "request_time": "2024-01-01T10:00:00Z",
  "strategy_name": "momentum_v1",
  "time_range_start": "2024-01-01",
  "time_range_end": "2024-12-31"
}
```

---

### 3.2 查询股票请求

**接口**: `GET /stock-requests/:id`

**示例**:

```bash
curl -i http://localhost:8001/stock-requests/1
```

**预期返回**:

- 存在: `200 OK` + JSON 数据
- 不存在: `404 Not Found`

---

### 3.3 删除股票请求

**接口**: `DELETE /stock-requests/:id`

**示例**:

```bash
curl -i -X DELETE http://localhost:8001/stock-requests/1
```

**预期返回**:

- 存在: `204 No Content`
- 不存在: `404 Not Found`

**注意**: 删除请求会级联删除关联的 `stock_request_stocks` 和 `stock_snapshots` 数据

---

## 4. 股票请求关联管理

### 4.1 创建请求-股票关联

**接口**: `POST /api/stock-request-stocks`

**请求体**:

```json
{
  "request_id": 1,
  "stock_code": "SH600519"
}
```

**示例**:

```bash
curl -i -X POST http://localhost:8001/api/stock-request-stocks \
  -H "Content-Type: application/json" \
  -d '{
    "request_id": 1,
    "stock_code": "SH600519"
  }'
```

**预期返回**:

- 成功: `201 Created`
- 外键错误/唯一性冲突: `400 Bad Request`

---

### 4.2 查询请求-股票关联

**接口**: `GET /api/stock-request-stocks/:request_id/:stock_code`

**示例**:

```bash
curl -i http://localhost:8001/api/stock-request-stocks/1/SH600519
```

**预期返回**:

- 存在: `200 OK` + JSON

```json
{
  "request_id": 1,
  "stock_code": "SH600519"
}
```

- 不存在: `404 Not Found`

---

### 4.3 删除请求-股票关联

**接口**: `DELETE /api/stock-request-stocks/:request_id/:stock_code`

**示例**:

```bash
curl -i -X DELETE http://localhost:8001/api/stock-request-stocks/1/SH600519
```

**预期返回**:

- 存在: `204 No Content`
- 不存在: `404 Not Found`

---

## 5. 股票快照管理

### 5.1 创建股票快照

**接口**: `POST /api/stock-snapshots`

**请求体**:

```json
{
  "request_id": 1,
  "stock_code": "SH600519",
  "stock_name": "贵州茅台",
  "latest_price": 1800.00,
  "change_pct": 2.50,
  "volume_ratio": 1.20,
  "turnover_rate": 0.8500,
  "bid_ask_ratio": 1.05,
  "main_force_inflow": 15000.00
}
```

**示例**:

```bash
curl -i -X POST http://localhost:8001/api/stock-snapshots \
  -H "Content-Type: application/json" \
  -d '{
    "request_id": 1,
    "stock_code": "SH600519",
    "stock_name": "贵州茅台",
    "latest_price": 1800.00,
    "change_pct": 2.50,
    "volume_ratio": 1.20,
    "turnover_rate": 0.8500,
    "bid_ask_ratio": 1.05,
    "main_force_inflow": 15000.00
  }'
```

**预期返回**:

- 成功: `201 Created` + `{"id": 1}`
- 外键错误: `400 Bad Request`

---

### 5.2 查询股票快照

**接口**: `GET /api/stock-snapshots/:id`

**示例**:

```bash
curl -i http://localhost:8001/api/stock-snapshots/1
```

**预期返回**:

- 存在: `200 OK` + JSON 数据

```json
{
  "id": 1,
  "request_id": 1,
  "stock_code": "SH600519",
  "stock_name": "贵州茅台",
  "latest_price": "1800.00",
  "change_pct": "2.50",
  "volume_ratio": "1.20",
  "turnover_rate": "0.8500",
  "bid_ask_ratio": "1.05",
  "main_force_inflow": "15000.00",
  "created_at": "2024-01-01T10:00:00Z"
}
```

- 不存在: `404 Not Found`

---

### 5.3 删除股票快照

**接口**: `DELETE /api/stock-snapshots/:id`

**示例**:

```bash
curl -i -X DELETE http://localhost:8001/api/stock-snapshots/1
```

**预期返回**:

- 存在: `204 No Content`
- 不存在: `404 Not Found`

---

## 6. 日K线数据管理

### 6.1 批量导入K线数据（从东方财富）

**接口**: `POST /api/daily-klines/import`

**功能**: 从东方财富网获取指定股票的K线数据并批量导入数据库

**请求体**:

```json
{
  "stock_code": "600519",
  "start_date": "20251201",
  "end_date": "20251227"
}
```

**字段说明**:

- `stock_code`: 股票代码（6位数字，如 `600519`）
- `start_date`: 开始日期（格式：YYYYMMDD）
- `end_date`: 结束日期（格式：YYYYMMDD）

**示例**:

```bash
# 导入贵州茅台 2025年12月的K线数据
curl -i -X POST http://localhost:8001/api/daily-klines/import \
  -H "Content-Type: application/json" \
  -d '{
    "stock_code": "600519",
    "start_date": "20251201",
    "end_date": "20251227"
  }'
```

**预期返回**:

- 成功: `200 OK` + JSON 数据

```json
{
  "success": true,
  "stock_code": "600519",
  "stock_name": "贵州茅台",
  "total_count": 18,
  "imported_count": 18,
  "failed_count": 0,
  "errors": []
}
```

- 部分成功（有重复数据）:

```json
{
  "success": true,
  "stock_code": "600519",
  "stock_name": "贵州茅台",
  "total_count": 18,
  "imported_count": 15,
  "failed_count": 0,
  "errors": [
    "Duplicate entry for 600519 on 2025-12-01",
    "Duplicate entry for 600519 on 2025-12-02",
    "Duplicate entry for 600519 on 2025-12-03"
  ]
}
```

- 失败: `400 Bad Request`

```json
{
  "error": "Failed to fetch kline data: ..."
}
```

**字段说明**:

- `success`: 是否成功（`failed_count == 0` 为 `true`）
- `stock_code`: 股票代码
- `stock_name`: 股票名称（从API返回）
- `total_count`: API返回的K线总数
- `imported_count`: 成功导入的数量
- `failed_count`: 导入失败的数量（不包括重复数据）
- `errors`: 错误和警告信息列表（包括重复数据提示）

**注意事项**:

- 重复数据（同一股票同一日期）不会导入失败，会记录在 `errors` 中但不计入 `failed_count`
- 导入过程会自动解析东方财富返回的K线数据
- K线数据格式：日期,开盘,收盘,最高,最低,成交量,成交额...

---

### 6.2 创建日K线数据

**接口**: `POST /api/daily-klines`

**请求体**:

```json
{
  "stock_code": "SH600519",
  "trade_date": "2024-01-02",
  "open_price": 1700.00,
  "high_price": 1820.00,
  "low_price": 1690.00,
  "close_price": 1805.00,
  "volume": 123456789,
  "amount": 99999999.99
}
```

**示例**:

```bash
curl -i -X POST http://localhost:8001/api/daily-klines \
  -H "Content-Type: application/json" \
  -d '{
    "stock_code": "SH600519",
    "trade_date": "2024-01-02",
    "open_price": 1700.00,
    "high_price": 1820.00,
    "low_price": 1690.00,
    "close_price": 1805.00,
    "volume": 123456789,
    "amount": 99999999.99
  }'
```

**预期返回**:

- 成功: `201 Created` + 完整 JSON 数据
- 唯一性冲突（同一股票同一日期）: `400 Bad Request`

---

### 6.3 查询日K线数据

**接口**: `GET /api/daily-klines/:stock_code/:trade_date`

**示例**:

```bash
curl -i http://localhost:8001/api/daily-klines/SH600519/2024-01-02
```

**预期返回**:

- 存在: `200 OK` + JSON 数据

```json
{
  "stock_code": "SH600519",
  "trade_date": "2024-01-02",
  "open_price": "1700.00",
  "high_price": "1820.00",
  "low_price": "1690.00",
  "close_price": "1805.00",
  "volume": 123456789,
  "amount": "99999999.99"
}
```

- 不存在: `404 Not Found`

---

### 6.4 删除日K线数据

**接口**: `DELETE /api/daily-klines/:stock_code/:trade_date`

**示例**:

```bash
curl -i -X DELETE http://localhost:8001/api/daily-klines/SH600519/2024-01-02
```

**预期返回**:

- 存在: `204 No Content`
- 不存在: `404 Not Found`

---

## 7. 盈利分析管理

### 7.1 创建盈利分析

**接口**: `POST /api/profit-analyses`

**请求体**:

```json
{
  "snapshot_id": 1,
  "strategy_name": "momentum_v1",
  "profit_rate": 5.25
}
```

**示例**:

```bash
curl -i -X POST http://localhost:8001/api/profit-analyses \
  -H "Content-Type: application/json" \
  -d '{
    "snapshot_id": 1,
    "strategy_name": "momentum_v1",
    "profit_rate": 5.25
  }'
```

**预期返回**:

- 成功: `201 Created` + `{"id": 1}`
- 外键错误（snapshot_id 不存在）: `400 Bad Request`

---

### 7.2 查询盈利分析

**接口**: `GET /api/profit-analyses/:id`

**示例**:

```bash
curl -i http://localhost:8001/api/profit-analyses/1
```

**预期返回**:

- 存在: `200 OK` + JSON 数据

```json
{
  "id": 1,
  "snapshot_id": 1,
  "strategy_name": "momentum_v1",
  "profit_rate": "5.25",
  "analysis_time": "2024-01-01T10:00:00Z"
}
```

- 不存在: `404 Not Found`

---

### 7.3 删除盈利分析

**接口**: `DELETE /api/profit-analyses/:id`

**示例**:

```bash
curl -i -X DELETE http://localhost:8001/api/profit-analyses/1
```

**预期返回**:

- 存在: `204 No Content`
- 不存在: `404 Not Found`

---

## 8. 定时任务管理

### 8.1 手动触发K线导入任务

**接口**: `POST /api/scheduler/trigger-kline-import`

**功能**: 手动触发K线导入定时任务，立即执行获取当天股票代码并批量导入K线数据的流程

**请求体**: 无需请求体

**示例**:

```bash
curl -i -X POST http://localhost:8001/api/scheduler/trigger-kline-import
```

**预期返回**:

- 成功: `200 OK` + JSON 数据

```json
{
  "success": true,
  "message": "K线导入任务执行完成，总计 3 只股票，成功 3 只，失败 0 只",
  "total_stocks": 3,
  "success_count": 3,
  "failed_count": 0,
  "details": [
    {
      "stock_code": "603819",
      "imported_count": 1,
      "success": true,
      "error": null
    },
    {
      "stock_code": "300991",
      "imported_count": 1,
      "success": true,
      "error": null
    },
    {
      "stock_code": "300107",
      "imported_count": 1,
      "success": true,
      "error": null
    }
  ]
}
```

- 部分失败:

```json
{
  "success": false,
  "message": "K线导入任务执行完成，总计 3 只股票，成功 2 只，失败 1 只",
  "total_stocks": 3,
  "success_count": 2,
  "failed_count": 1,
  "details": [
    {
      "stock_code": "603819",
      "imported_count": 1,
      "success": true,
      "error": null
    },
    {
      "stock_code": "300991",
      "imported_count": 0,
      "success": false,
      "error": "Failed to fetch kline data: ..."
    },
    {
      "stock_code": "300107",
      "imported_count": 1,
      "success": true,
      "error": null
    }
  ]
}
```

- 有数据被跳过（已存在）:

```json
{
  "success": true,
  "message": "K线导入任务执行完成，总计 3 只股票，成功 3 只，失败 0 只",
  "total_stocks": 3,
  "success_count": 3,
  "failed_count": 0,
  "details": [
    {
      "stock_code": "603819",
      "imported_count": 1,
      "success": true,
      "error": null
    },
    {
      "stock_code": "300991",
      "imported_count": 0,
      "success": true,
      "error": "数据已存在，跳过导入"
    },
    {
      "stock_code": "300107",
      "imported_count": 1,
      "success": true,
      "error": null
    }
  ]
}
```

- 无股票代码:

```json
{
  "success": true,
  "message": "K线导入任务执行完成，总计 0 只股票，成功 0 只，失败 0 只",
  "total_stocks": 0,
  "success_count": 0,
  "failed_count": 0,
  "details": []
}
```

**字段说明**:

- `success`: 是否全部成功（`failed_count == 0`）
- `message`: 执行结果描述
- `total_stocks`: 处理的股票总数
- `success_count`: 成功导入的股票数
- `failed_count`: 失败的股票数
- `details`: 每只股票的详细导入情况
  - `stock_code`: 股票代码
  - `imported_count`: 实际导入的K线记录数
  - `success`: 该股票是否导入成功
  - `error`: 错误信息（如果失败）

**注意事项**:

- 该接口会立即执行K线导入任务，不受定时任务时间限制
- 导入逻辑与定时任务完全相同（每天15:01自动执行）
- 适用于测试、补录数据或在定时任务时间外手动执行
- **智能日期处理**：如果是周末，会自动回溯到上周五获取数据
- **智能去重**：导入前会检查数据库，如果已有当天数据则自动跳过，避免重复导入

---

### 8.2 手动触发盈利分析任务

**接口**: `POST /api/scheduler/trigger-profit-analysis`

**功能**: 手动触发盈利分析定时任务，分析昨日入库的股票快照与今日K线数据的盈利情况

**请求体**: 无需请求体

**示例**:

```bash
curl -i -X POST http://localhost:8001/api/scheduler/trigger-profit-analysis
```

**预期返回**:

- 成功: `200 OK` + JSON 数据

```json
{
  "success": true,
  "message": "盈利分析任务执行完成，总计 3 个快照，分析 3 个，跳过 0 个，无K线 0 个",
  "total_snapshots": 3,
  "analyzed_count": 3,
  "skipped_count": 0,
  "no_kline_count": 0,
  "details": [
    {
      "stock_code": "603819",
      "stock_name": "神力股份",
      "profit_rate": 2,
      "success": true,
      "error": null
    },
    {
      "stock_code": "300991",
      "stock_name": "创益通",
      "profit_rate": 1,
      "success": true,
      "error": null
    },
    {
      "stock_code": "300107",
      "stock_name": "建新股份",
      "profit_rate": 0,
      "success": true,
      "error": null
    }
  ]
}
```

- 有快照被跳过（已分析或无K线）:

```json
{
  "success": true,
  "message": "盈利分析任务执行完成，总计 3 个快照，分析 1 个，跳过 1 个，无K线 1 个",
  "total_snapshots": 3,
  "analyzed_count": 1,
  "skipped_count": 1,
  "no_kline_count": 1,
  "details": [
    {
      "stock_code": "603819",
      "stock_name": "神力股份",
      "profit_rate": 2,
      "success": true,
      "error": null
    },
    {
      "stock_code": "300991",
      "stock_name": "创益通",
      "profit_rate": -1,
      "success": true,
      "error": "分析记录已存在，跳过"
    },
    {
      "stock_code": "300107",
      "stock_name": "建新股份",
      "profit_rate": -1,
      "success": true,
      "error": "2025-12-28的K线数据不存在"
    }
  ]
}
```

**字段说明**:

- `success`: 是否成功执行
- `message`: 执行结果描述
- `total_snapshots`: 处理的快照总数
- `analyzed_count`: 成功分析的快照数
- `skipped_count`: 跳过的快照数（已有分析记录）
- `no_kline_count`: 因无K线数据跳过的快照数
- `details`: 每个快照的详细分析情况
  - `stock_code`: 股票代码
  - `stock_name`: 股票名称
  - `profit_rate`: 盈利指标（0/1/2，-1表示未分析）
  - `success`: 是否处理成功
  - `error`: 错误/提示信息

**盈利指标说明**:

```
入场价 = stock_snapshots.latest_price
profit_high = 入场价 × 1.10 (原价+10%)
profit_low = 入场价 × 1.05 (原价+5%)

判断规则：
├─ high >= profit_high 且 close >= profit_low → profit_rate = 2
├─ high >= profit_low 但 close < profit_low → profit_rate = 1
└─ high < profit_low → profit_rate = 0
```

**注意事项**:

- 该接口会立即执行盈利分析任务，不受定时任务时间限制
- 分析逻辑与定时任务完全相同（每天15:30自动执行）
- 对应的 `stock_requests.time_range_end` 必须为空（未处理）
- **K线日期计算**：使用 `stock_requests.time_range_start + 1 天` 作为K线查询日期
- **智能日期处理**：如果 time_range_start + 1 是周末，会顺延到周一
- **智能去重**：已存在分析记录的快照会自动跳过
- 分析完成后会自动更新 `stock_requests.time_range_end` 标记处理完成

---

## 附录：错误码说明

| 状态码                        | 说明                                        |
| ----------------------------- | ------------------------------------------- |
| `200 OK`                    | 请求成功，返回数据                          |
| `201 Created`               | 资源创建成功                                |
| `204 No Content`            | 删除成功，无返回内容                        |
| `400 Bad Request`           | 请求参数错误、数据库约束冲突（外键/唯一键） |
| `404 Not Found`             | 资源不存在                                  |
| `500 Internal Server Error` | 服务器内部错误（数据库连接失败等）          |
| `502 Bad Gateway`           | 上游服务错误（仅限 EastMoney 接口）         |

---

## 附录：数据库关系说明

```
stock_requests (请求主表)
    ├── stock_request_stocks (请求关联的股票代码)
    └── stock_snapshots (股票快照数据)
            └── profit_analysis (基于快照的盈利分析)

daily_klines (独立的日K线表)
```

**级联删除关系**:

- 删除 `stock_requests` → 级联删除关联的 `stock_request_stocks` 和 `stock_snapshots`
- 删除 `stock_snapshots` → 级联删除关联的 `profit_analysis`

---

## 测试工作流示例

### 工作流 1: 股票筛选 + 盈利分析

```bash
# 1. 健康检查
curl http://localhost:8001/healthz

# 2. 创建股票请求
curl -X POST http://localhost:8001/stock-requests \
  -H "Content-Type: application/json" \
  -d '{"strategy_name":"test_strategy"}'
# 返回: {"id": 1, "request_uuid": "...", ...}

# 3. 创建股票快照（使用上面返回的 request_id）
curl -X POST http://localhost:8001/api/stock-snapshots \
  -H "Content-Type: application/json" \
  -d '{
    "request_id": 1,
    "stock_code": "SH600519",
    "stock_name": "贵州茅台",
    "latest_price": 1800.00,
    "change_pct": 2.50,
    "volume_ratio": 1.20,
    "turnover_rate": 0.8500,
    "bid_ask_ratio": 1.05,
    "main_force_inflow": 15000.00
  }'
# 返回: {"id": 1}

# 4. 创建盈利分析（使用上面返回的 snapshot_id）
curl -X POST http://localhost:8001/api/profit-analyses \
  -H "Content-Type: application/json" \
  -d '{
    "snapshot_id": 1,
    "strategy_name": "momentum_v1",
    "profit_rate": 5.25
  }'
# 返回: {"id": 1}

# 5. 查询盈利分析
curl http://localhost:8001/api/profit-analyses/1

# 6. 条件筛选股票（自动落库）
curl "http://localhost:8001/api/stock/filtered/param?limit=10"
# 该接口会自动创建 stock_request 并插入 stock_snapshots
```

### 工作流 2: K线数据批量导入

```bash
# 1. 导入贵州茅台最近一个月的K线数据
curl -X POST http://localhost:8001/api/daily-klines/import \
  -H "Content-Type: application/json" \
  -d '{
    "stock_code": "600519",
    "start_date": "20251201",
    "end_date": "20251227"
  }'
# 返回: {"success": true, "imported_count": 18, ...}

# 2. 查询指定日期的K线数据
curl http://localhost:8001/api/daily-klines/600519/2025-12-27

# 3. 查询单只股票实时数据
curl "http://localhost:8001/api/stock?code=600519&source=em"

# 4. 删除指定日期的K线数据
curl -X DELETE http://localhost:8001/api/daily-klines/600519/2025-12-27
```

### 工作流 3: 手动触发K线导入任务

```bash
# 1. 先筛选并入库一些股票
curl "http://localhost:8001/api/stock/filtered/param?limit=5"

# 2. 手动触发K线导入任务
curl -X POST http://localhost:8001/api/scheduler/trigger-kline-import
# 返回: {"success": true, "total_stocks": 5, "success_count": 5, ...}

# 3. 查询导入的K线数据
curl http://localhost:8001/api/daily-klines/600519/2025-12-28
```

### 工作流 4: 完整盈利分析流程

```bash
# ========== 第一天：筛选入库股票 ==========

# 1. 使用筛选接口入库股票（自动创建 stock_request 和 stock_snapshots）
curl "http://localhost:8001/api/stock/filtered/param?limit=5"
# 返回: {"count": 5, "items": [...]}
# 此时 stock_requests.time_range_end 为空

# ========== 第二天：执行盈利分析 ==========

# 2. 先确保今日K线数据已导入
curl -X POST http://localhost:8001/api/scheduler/trigger-kline-import
# 返回: {"success": true, "total_stocks": 5, ...}

# 3. 手动触发盈利分析任务
curl -X POST http://localhost:8001/api/scheduler/trigger-profit-analysis
# 返回: {"success": true, "total_snapshots": 5, "analyzed_count": 5, ...}

# 4. 查询盈利分析结果
curl http://localhost:8001/api/profit-analyses/1
# 返回: {"id": 1, "snapshot_id": 1, "strategy_name": "OHLC", "profit_rate": "2.00", ...}
```

**盈利分析流程说明**:

1. **第一天（工作日）**：使用 `/api/stock/filtered/param` 筛选股票
   - 系统自动创建 `stock_request` 记录
   - `time_range_start` 设置为当天日期，`time_range_end` 为空
   - 系统自动将筛选结果存入 `stock_snapshots`，记录入场价

2. **第二天（工作日）15:30**：定时任务自动执行盈利分析
   - 查找 `time_range_end` 为空的请求
   - 获取每个请求下的所有快照（通过 `request_id` 关联）
   - **K线日期 = time_range_start + 1 天**（周末顺延到周一）
   - 用快照的入场价与该日K线的最高价/收盘价比较
   - 计算 `profit_rate`（0/1/2）并写入 `profit_analysis`
   - 更新 `time_range_end` 标记处理完成

3. **盈利指标含义**:
   - `profit_rate = 2`: 高收益（最高价≥+10%，收盘价≥+5%）
   - `profit_rate = 1`: 中收益（最高价≥+5%，收盘价<+5%）
   - `profit_rate = 0`: 低收益（最高价<+5%）

---

**最后更新**: 2025-12-28
**后端版本**: rust-backend v0.1.0

## 附录：重构说明

### K线导入接口架构

K线批量导入接口 (`POST /api/daily-klines/import`) 采用分层架构设计：

```
Handler 层 (daily_kline.rs)
  ↓ 调用
Utils 层 (http_client.rs) - 创建 HTTP 客户端
  ↓
Service 层 (kline_service.rs) - 业务逻辑
  ├─ fetch_eastmoney_kline() - 从东方财富获取数据
  ├─ parse_kline_json() - 解析 JSON 响应
  └─ fetch_and_parse_kline_data() - 完整流程
  ↓
Handler 层 - 批量插入数据库
  ↓
Repository 层 (daily_kline.rs) - 数据持久化
```

**优势**：

- 职责分离，易于维护
- HTTP 客户端可复用
- 业务逻辑独立，便于测试
- 数据解析与网络请求分离
