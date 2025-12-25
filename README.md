# Axum 最小后端（rust-backend）

### 简介
最小可运行的 Axum 学习后端，提供基础路由、CORS 与访问日志，便于后续将 FastAPI 迁移到 Axum。

### 运行要求
- Rust（stable）。安装参考 `https://www.rust-lang.org/tools/install`

### 快速开始（本地）
```powershell
cd rust-backend
$env:RUST_LOG='info,tower_http=info,axum=info'; cargo run
```
默认监听：`http://127.0.0.1:8001`

终止运行：在运行窗口按 Ctrl+C（若端口仍占用，见“故障排查”）。

### 接口
- GET `/` → 纯文本：`Axum minimal backend`
- GET `/healthz` → 纯文本：`ok`
- GET `/api/hello` → JSON：`{ "message": "hello, axum" }`
- GET `/api/time` → JSON：`{ "epoch_ms": <当前毫秒> }`

### 配置
- 环境变量（可通过 `.env` 覆盖，项目已启用 `dotenvy`）：
  - `HOST`（默认 `127.0.0.1`）
  - `PORT`（默认 `8001`）
  - `ALLOWED_ORIGINS`（逗号分隔，默认允许 `http://localhost:5173` 与 `http://127.0.0.1:5173`）

示例（PowerShell 临时设置）：
```powershell
$env:HOST='127.0.0.1'; $env:PORT='8001'; cargo run
```

### 日志
- 使用 `tracing` 与 `tower-http` 的 `TraceLayer`，默认 INFO 级别输出请求开始与结束（含状态码、耗时），错误为 ERROR 级别。
- 建议启动时设置：
```powershell
$env:RUST_LOG='info,tower_http=info,axum=info'; cargo run
```

### CORS
- 默认允许：`http://localhost:5173`、`http://127.0.0.1:5173`
- 可通过 `ALLOWED_ORIGINS` 逗号分隔覆盖。

### 构建
```powershell
cd rust-backend
cargo build --release
```
产物：`rust-backend/target/release/rust-backend(.exe)`

### 故障排查（Windows）
- 构建时报 “拒绝访问 (os error 5)” 多因可执行仍在运行或被占用：
```powershell
Get-Process rust-backend -ErrorAction SilentlyContinue | Stop-Process -Force
```
- 端口占用（默认 8001）：
```powershell
netstat -ano | findstr :8001
taskkill /PID <PID> /F
```


