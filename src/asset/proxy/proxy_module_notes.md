# 代理模块说明与问题复盘

## 模块目的
- 提供带认证的代理请求能力，用于访问东方财富接口。
- 缓存代理 IP，避免频繁向代理服务请求新 IP。
- 在请求失败时自动失效并切换代理。

## 关键文件
- `src/utils/proxy/client.rs`：`ProxyClient` 的核心实现与缓存逻辑
- `src/utils/proxy/http.rs`：通过代理发送请求并处理错误
- `src/utils/proxy/mod.rs`：模块导出
- `src/services/stock_plate_em.rs`：板块接口调用（通过代理）

## 核心流程（简化）
1. 获取 `ProxyClient`（共享实例）
2. `ProxyClient::ensure_proxy_client`：
   - 缓存未过期：复用已有 `Client`
   - 缓存过期或不存在：请求代理服务获取新 IP，并构建新的 `Client`
3. 通过 `proxy_get_json` 发送请求
4. 请求失败则 `invalidate_proxy`，下一次会重新拉取代理

## 缓存与失效策略
- 缓存字段：`CachedProxy { client, deadline }`
- 过期判断：`Local::now() >= deadline`
- 失败失效：HTTP 错误或非 2xx 响应会触发 `invalidate_proxy`

## 最近问题复盘
### 现象
- 代理 IP 提取频率限制为 60 次/分钟
- 板块同步并发提高到 100 后，代理服务被大量调用，触发限流或失败

### 根因
- `fetch_em_plate_list` 之前每次调用都会 `ProxyClient::from_env()` 新建实例
- 新实例没有共享缓存，导致每个请求都拉一次代理 IP
- 并发提升后，代理提取请求瞬间超过限制

## 解决方案（已实施）
- 在代理模块内加入共享 `ProxyClient`
- `shared_proxy_client()` 返回 `Arc<Mutex<ProxyClient>>`，全局复用缓存
- 板块同步任务与接口调用统一使用共享实例

## 相关配置
- `PROXY_API_URL`：代理服务地址
- `PROXY_AUTH_KEY` / `PROXY_AUTH_PWD`：代理认证
- `PROXY_MAX_RETRIES`：代理接口重试次数
- `PROXY_TIMEOUT_SECS`：代理接口超时秒数

## 注意事项
- 高并发下应优先保证代理缓存复用，避免触发代理提取频率限制
- 若代理服务返回短 TTL，可能仍会导致频繁切换，需要结合服务端策略调整
