# 东方财富板块接口测试（curl）

**基础地址**: `http://localhost:8001`  
**API 前缀**: `/api`  
**Content-Type**: `application/json`

---

## 1. 获取东方财富板块列表

**接口**: `GET /api/stock-plates/em`

```bash
curl -i "http://localhost:8001/api/stock-plates/em?stock_code=600519"
```

**预期返回**: `200 OK` + JSON

```json
{
  "total": 20,
  "items": [
    {
      "plate_code": "BK0477",
      "name": "酿酒行业"
    },
    {
      "plate_code": "BK0173",
      "name": "贵州板块"
    }
  ]
}
```

---

## 2. 说明

- 数据来源：东方财富接口 `push2.eastmoney.com/api/qt/slist/get`
- 解析字段：`diff[].f12 -> plate_code`，`diff[].f14 -> name`
- 该接口只做数据拉取与解析，不入库
- 必填参数：`stock_code`（示例：`600519`）
