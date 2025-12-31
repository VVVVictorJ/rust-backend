# 交易日期查询接口测试指南

## 接口信息

**端点**: `POST /api/stock-trade-date-query`

**功能**: 根据交易日期查询当日有主力资金流入（main_force_inflow > 0）的股票快照数据，支持分页。

## 请求示例

### 基本请求

```bash
curl -X POST http://localhost:8001/api/stock-trade-date-query \
  -H "Content-Type: application/json" \
  -d '{"trade_date": "2025-12-29", "page": 1, "page_size": 20}'
```

### 请求参数说明

| 参数       | 类型    | 必填 | 默认值 | 说明                       |
| ---------- | ------- | ---- | ------ | -------------------------- |
| trade_date | string  | 是   | -      | 交易日期，格式：YYYY-MM-DD |
| page       | integer | 否   | 1      | 页码，从 1 开始            |
| page_size  | integer | 否   | 20     | 每页数量，范围：1-100      |

## 响应示例

### 成功响应 (200 OK)

```json
{
  "data": [
    {
      "stock_code": "000001",
      "stock_name": "平安银行",
      "latest_price": "12.35",
      "close_price": "12.28",
      "change_pct": "0.57",
      "volume_ratio": "1.23",
      "turnover_rate": "0.45",
      "bid_ask_ratio": "1.15",
      "main_force_inflow": "123456789.00",
      "created_at": "2025-12-29T08:30:00Z"
    }
  ],
  "total": 150,
  "page": 1,
  "page_size": 20,
  "total_pages": 8
}
```

### 响应字段说明

| 字段                     | 类型     | 说明                  |
| ------------------------ | -------- | --------------------- |
| data                     | array    | 股票数据列表          |
| data[].stock_code        | string   | 股票代码              |
| data[].stock_name        | string   | 股票名称              |
| data[].latest_price      | decimal  | 最新价格              |
| data[].close_price       | decimal  | 收盘价（可能为 null） |
| data[].change_pct        | decimal  | 涨跌幅                |
| data[].volume_ratio      | decimal  | 量比                  |
| data[].turnover_rate     | decimal  | 换手率                |
| data[].bid_ask_ratio     | decimal  | 委比                  |
| data[].main_force_inflow | decimal  | 主力资金流入          |
| data[].created_at        | datetime | 创建时间              |
| total                    | integer  | 总记录数              |
| page                     | integer  | 当前页码              |
| page_size                | integer  | 每页数量              |
| total_pages              | integer  | 总页数                |

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

## 查询特性

1. **主力资金过滤**: 只返回主力资金流入大于 0 的股票
2. **日期匹配**: 股票快照的创建日期必须与 K 线数据的交易日期匹配
3. **排序**: 结果按主力资金流入金额降序排列
4. **分页**: 支持分页查询，避免一次性返回大量数据

## 测试场景

### 场景 1: 查询指定日期的第一页数据

```bash
curl -X POST http://localhost:8080/api/stock-trade-date-query \
  -H "Content-Type: application/json" \
  -d '{
    "trade_date": "2025-12-29"
  }'
```

### 场景 2: 查询指定日期的第 2 页数据

```bash
curl -X POST http://localhost:8080/api/stock-trade-date-query \
  -H "Content-Type: application/json" \
  -d '{
    "trade_date": "2025-12-29",
    "page": 2,
    "page_size": 50
  }'
```

### 场景 3: 错误的日期格式

```bash
curl -X POST http://localhost:8080/api/stock-trade-date-query \
  -H "Content-Type: application/json" \
  -d '{
    "trade_date": "2025/12/29"
  }'
```

预期返回 400 错误。

## 注意事项

1. 确保数据库中存在对应日期的股票快照和 K 线数据
2. 交易日期必须是有效的交易日，非交易日可能返回空结果
3. 分页参数如果超出范围，会返回 400 错误
4. 建议合理设置 page_size，避免单次查询数据量过大
