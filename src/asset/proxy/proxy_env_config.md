# Proxy 环境变量配置说明

本文档说明代理模块涉及的环境变量，以及开发环境（dev）和生产环境（prod）的推荐配置方式。

## 环境变量清单

### 代理相关

以下变量由 `src/utils/proxy/client.rs` 读取：

- `PROXY_API_URL`  
  代理服务的拉取地址。未配置时使用默认值（代码内置的默认 API URL）。

- `PROXY_AUTH_KEY`  
  代理认证用户名/Key（必填）。

- `PROXY_AUTH_PWD`  
  代理认证密码（必填）。

- `PROXY_MAX_RETRIES`  
  拉取代理 IP 的重试次数（可选，默认 3）。

- `PROXY_TIMEOUT_SECS`  
  代理请求超时（秒，可选，默认 15）。

### 数据库相关

以下变量与数据库连接/写入并发相关（见 `.env` 示例）：

- `DATABASE_URL`  
  数据库连接字符串（必填）。示例：`postgres://user:password@host/db_name`

- `DB_WRITE_CONCURRENCY`  
  数据库写入并发上限（可选，默认 20），用于控制高并发任务的写库压力。

## Dev 环境建议

目标：开发环境通常并发较低，重点是稳定与方便调试。

推荐值：
- `PROXY_API_URL`：使用开发/测试代理服务地址
- `PROXY_AUTH_KEY` / `PROXY_AUTH_PWD`：使用测试账号
- `PROXY_MAX_RETRIES=3`（默认即可）
- `PROXY_TIMEOUT_SECS=15`（默认即可）

示例（`.env` 或本地环境变量）：
```
PROXY_API_URL=https://your-dev-proxy.example.com/get?key=xxxx&num=1&area=310000&distinct=true
PROXY_AUTH_KEY=dev_key
PROXY_AUTH_PWD=dev_pwd
PROXY_MAX_RETRIES=3
PROXY_TIMEOUT_SECS=15
DATABASE_URL=postgres://stock_user:password@localhost/stock_analysis
DB_WRITE_CONCURRENCY=20
```

## Prod 环境建议

目标：生产环境高并发，避免频繁触发代理提取限制，优先保障稳定性。

推荐值：
- `PROXY_API_URL`：生产代理服务地址
- `PROXY_AUTH_KEY` / `PROXY_AUTH_PWD`：生产账号
- `PROXY_MAX_RETRIES=3~5`（代理服务波动时可适当提高）
- `PROXY_TIMEOUT_SECS=15~20`（视代理服务稳定性调整）

示例（部署环境变量）：
```
PROXY_API_URL=https://your-prod-proxy.example.com/get?key=xxxx&num=1&area=310000&distinct=true
PROXY_AUTH_KEY=prod_key
PROXY_AUTH_PWD=prod_pwd
PROXY_MAX_RETRIES=3
PROXY_TIMEOUT_SECS=20
DATABASE_URL=postgres://stock_user:password@prod-host/stock_analysis
DB_WRITE_CONCURRENCY=100
```

## 注意事项

- 强烈建议使用共享代理缓存（`shared_proxy_client`）以降低代理提取频率。
- 若代理服务的 IP TTL 很短，可考虑：
  - 降低并发峰值
  - 适当增加 `PROXY_TIMEOUT_SECS`
  - 与代理服务方确认更高的频率上限或更长 TTL

