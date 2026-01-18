# 股票板块 / 股票表 / 关系表接口测试（curl）

**基础地址**: `http://localhost:8001`  
**API 前缀**: `/api`  
**Content-Type**: `application/json`

---

## 1. stock_plate（板块）

### 1.1 新增板块

**接口**: `POST /api/stock-plates`

```bash
curl -i -X POST http://localhost:8001/api/stock-plates \
  -H "Content-Type: application/json" \
  -d '{
    "plate_code": "BK001",
    "name": "半导体"
  }'
```

**预期返回**: `201 Created` + JSON（包含 `id`）

### 1.2 查询板块

**接口**: `GET /api/stock-plates/:id`

```bash
curl -i http://localhost:8001/api/stock-plates/1
```

**预期返回**: `200 OK` 或 `404 Not Found`

### 1.3 列表板块

**接口**: `GET /api/stock-plates`

```bash
curl -i http://localhost:8001/api/stock-plates
```

**预期返回**: `200 OK` + JSON 数组

### 1.4 更新板块

**接口**: `PUT /api/stock-plates/:id`

```bash
curl -i -X PUT http://localhost:8001/api/stock-plates/1 \
  -H "Content-Type: application/json" \
  -d '{
    "plate_code": "BK001",
    "name": "半导体-更新"
  }'
```

**预期返回**: `200 OK` + JSON

### 1.5 删除板块

**接口**: `DELETE /api/stock-plates/:id`

```bash
curl -i -X DELETE http://localhost:8001/api/stock-plates/1
```

**预期返回**: `204 No Content` 或 `404 Not Found`

---

## 2. stock_table（股票）

### 2.1 新增股票

**接口**: `POST /api/stock-tables`

```bash
curl -i -X POST http://localhost:8001/api/stock-tables \
  -H "Content-Type: application/json" \
  -d '{
    "stock_code": "600519",
    "stock_name": "贵州茅台"
  }'
```

**预期返回**: `201 Created` + JSON（包含 `id`）

### 2.2 查询股票

**接口**: `GET /api/stock-tables/:id`

```bash
curl -i http://localhost:8001/api/stock-tables/1
```

**预期返回**: `200 OK` 或 `404 Not Found`

### 2.3 列表股票

**接口**: `GET /api/stock-tables`

```bash
curl -i http://localhost:8001/api/stock-tables
```

**预期返回**: `200 OK` + JSON 数组

### 2.4 更新股票

**接口**: `PUT /api/stock-tables/:id`

```bash
curl -i -X PUT http://localhost:8001/api/stock-tables/1 \
  -H "Content-Type: application/json" \
  -d '{
    "stock_code": "600519",
    "stock_name": "贵州茅台-更新"
  }'
```

**预期返回**: `200 OK` + JSON

### 2.5 删除股票

**接口**: `DELETE /api/stock-tables/:id`

```bash
curl -i -X DELETE http://localhost:8001/api/stock-tables/1
```

**预期返回**: `204 No Content` 或 `404 Not Found`

---

## 3. stock_plate_stock_table（板块-股票关系）

### 3.1 新增关系

**接口**: `POST /api/stock-plate-stocks`

```bash
curl -i -X POST http://localhost:8001/api/stock-plate-stocks \
  -H "Content-Type: application/json" \
  -d '{
    "plate_id": 1,
    "stock_table_id": 1
  }'
```

**预期返回**: `201 Created`

### 3.2 分页查询关系（可按板块名过滤）

**接口**: `GET /api/stock-plate-stocks`

**参数**:
- `plate_name`: 过滤板块名（可选，模糊匹配）
- `page`: 页码，从 1 开始（默认 1）
- `page_size`: 每页数量（默认 20，范围 1-100）

```bash
curl -i "http://localhost:8001/api/stock-plate-stocks?plate_name=半导体&page=1&page_size=20"
```

**预期返回**: `200 OK` + JSON

```json
{
  "data": [
    {
      "plate_id": 1,
      "plate_name": "半导体",
      "stock_table_id": 1,
      "stock_code": "600519",
      "stock_name": "贵州茅台"
    }
  ],
  "total": 1,
  "page": 1,
  "page_size": 20,
  "total_pages": 1
}
```

### 3.3 删除关系

**接口**: `DELETE /api/stock-plate-stocks/:plate_id/:stock_table_id`

```bash
curl -i -X DELETE http://localhost:8001/api/stock-plate-stocks/1/1
```

**预期返回**: `204 No Content` 或 `404 Not Found`

---

## 4. 建议测试流程

```bash
# 1. 创建板块
curl -i -X POST http://localhost:8001/api/stock-plates \
  -H "Content-Type: application/json" \
  -d '{"plate_code":"BK001","name":"半导体"}'

# 2. 创建股票
curl -i -X POST http://localhost:8001/api/stock-tables \
  -H "Content-Type: application/json" \
  -d '{"stock_code":"600519","stock_name":"贵州茅台"}'

# 3. 创建关系
curl -i -X POST http://localhost:8001/api/stock-plate-stocks \
  -H "Content-Type: application/json" \
  -d '{"plate_id":1,"stock_table_id":1}'

# 4. 关系查询（带过滤）
curl -i "http://localhost:8001/api/stock-plate-stocks?plate_name=半导体&page=1&page_size=20"

# 5. 删除关系
curl -i -X DELETE http://localhost:8001/api/stock-plate-stocks/1/1
```
