# 价格对比查询接口测试指南

## 接口信息

**端点**: `POST /api/stock-price-compare`

**功能**: 根据交易日期查询盈利等级为 A/B 的股票快照与价格对比数据，自动处理节假日，支持分页。

## 请求示例

### 基本请求

```bash
curl -X POST http://localhost:8001/api/stock-price-compare \
  -H "Content-Type: application/json" \
  -d '{
    "trade_date": "2025-12-30",
    "page": 1,
    "page_size": 20
  }'
```

### 请求参数说明

| 参数 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|--------|------|
| trade_date | string | 是 | - | 交易日期，格式：YYYY-MM-DD |
| page | integer | 否 | 1 | 页码，从 1 开始 |
| page_size | integer | 否 | 20 | 每页数量，范围：1-100 |

## 响应示例

### 成功响应 (200 OK)

```json
{
  "data": [
    {
      "stock_code": "000001",
      "stock_name": "平安银行",
      "latest_price": "12.35",
      "high_price": "12.50",
      "close_price": "12.28",
      "open_price": "12.20",
      "low_price": "12.15",
      "grade": "A",
      "created_at": "2025-12-29T08:30:00Z"
    },
    {
      "stock_code": "000002",
      "stock_name": "万科A",
      "latest_price": "8.56",
      "high_price": "8.78",
      "close_price": "8.65",
      "open_price": "8.50",
      "low_price": "8.45",
      "grade": "B",
      "created_at": "2025-12-29T08:30:00Z"
    }
  ],
  "total": 50,
  "page": 1,
  "page_size": 20,
  "total_pages": 3,
  "snapshot_date": "2025-12-29",
  "trade_date": "2025-12-30"
}
```

### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| data | array | 股票数据列表 |
| data[].stock_code | string | 股票代码 |
| data[].stock_name | string | 股票名称 |
| data[].latest_price | decimal | 快照最新价格 |
| data[].high_price | decimal | 交易日最高价 |
| data[].close_price | decimal | 交易日收盘价 |
| data[].open_price | decimal | 交易日开盘价 |
| data[].low_price | decimal | 交易日最低价 |
| data[].grade | string | 盈利等级（A/B） |
| data[].created_at | datetime | 快照创建时间 |
| total | integer | 总记录数 |
| page | integer | 当前页码 |
| page_size | integer | 每页数量 |
| total_pages | integer | 总页数 |
| snapshot_date | string | 快照日期（前一个交易日） |
| trade_date | string | 查询的交易日期 |

## 错误响应

### 日期格式错误 (400 Bad Request)

```json
{
  "error": "bad request",
  "message": "Invalid date format, expected YYYY-MM-DD"
}
```

### 分页参数错误 (400 Bad Request)

```json
{
  "error": "bad request",
  "message": "page_size must be between 1 and 100"
}
```

### 服务器错误 (500 Internal Server Error)

```json
{
  "error": "internal server error"
}
```

## 特殊场景

### 场景 1: 找不到前一个交易日

如果查询的日期之前没有交易日数据（如数据库中的第一个交易日），接口会返回空结果：

```json
{
  "data": [],
  "total": 0,
  "page": 1,
  "page_size": 20,
  "total_pages": 0,
  "snapshot_date": null,
  "trade_date": "2025-01-02"
}
```

### 场景 2: 周末和节假日处理

系统会自动跳过周末和节假日，查询最近的实际交易日：

```bash
# 查询 2025-01-02（周一）的数据
# 系统会自动找到上周五 2024-12-29 作为 snapshot_date
curl -X POST http://localhost:8001/api/stock-price-compare \
  -H "Content-Type: application/json" \
  -d '{
    "trade_date": "2025-01-02"
  }'
```

### 场景 3: 长假后第一天

```bash
# 查询春节后第一个交易日
# 系统会自动找到春节前最后一个交易日作为 snapshot_date
curl -X POST http://localhost:8001/api/stock-price-compare \
  -H "Content-Type: application/json" \
  -d '{
    "trade_date": "2025-02-03"
  }'
```

## 节假日处理逻辑

1. 系统通过查询 `daily_klines` 表获取实际的交易日期
2. 自动找到指定日期之前最近的一个交易日作为快照日期
3. 无需维护节假日列表，完全基于数据库中的实际数据
4. 处理所有类型的非交易日：周末、法定节假日、调休等

## 盈利等级说明

| Grade | profit_rate | 说明 |
|-------|-------------|------|
| A | 2 | 高盈利潜力 |
| B | 1 | 中等盈利潜力 |
| C | 其他 | 低盈利潜力（不在此接口返回） |

**注意**: 此接口只返回 Grade 为 A 或 B 的股票。

## 测试场景

### 场景 1: 查询指定日期的第一页数据

```bash
curl -X POST http://localhost:8001/api/stock-price-compare \
  -H "Content-Type: application/json" \
  -d '{
    "trade_date": "2025-12-30"
  }'
```

### 场景 2: 查询指定日期的第 2 页数据

```bash
curl -X POST http://localhost:8001/api/stock-price-compare \
  -H "Content-Type: application/json" \
  -d '{
    "trade_date": "2025-12-30",
    "page": 2,
    "page_size": 50
  }'
```

### 场景 3: 错误的日期格式

```bash
curl -X POST http://localhost:8001/api/stock-price-compare \
  -H "Content-Type: application/json" \
  -d '{
    "trade_date": "2025/12/30"
  }'
```

预期返回 400 错误。

### 场景 4: 分页参数超出范围

```bash
curl -X POST http://localhost:8001/api/stock-price-compare \
  -H "Content-Type: application/json" \
  -d '{
    "trade_date": "2025-12-30",
    "page": 1,
    "page_size": 150
  }'
```

预期返回 400 错误。

## 数据依赖

此接口依赖以下数据表：

1. **profit_analysis** - 盈利分析数据
2. **stock_snapshots** - 股票快照数据
3. **daily_klines** - 日线数据

确保这些表中有对应日期的数据才能正常查询。

## 注意事项

1. 交易日期必须是有效的交易日，系统会自动查找前一个交易日
2. 如果数据库中没有前一个交易日的数据，会返回空结果（total=0）
3. 返回结果使用 DISTINCT 去重，确保每个股票只出现一次
4. 建议合理设置 page_size，避免单次查询数据量过大
5. Grade 为字符串类型："A"、"B"

