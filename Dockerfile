# syntax=docker/dockerfile:1

# Build
FROM rust:1.80-slim AS builder
WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates build-essential && \
    rm -rf /var/lib/apt/lists/*

# 仅复制清单以命中依赖缓存
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs && \
    cargo build --release && rm -rf target/release/deps/rust_backend*

# 复制源码并编译
COPY . ./
RUN cargo build --release

# Runtime
FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/* && update-ca-certificates

ENV RUST_LOG=info,tower_http=info,axum=info
ENV HOST=0.0.0.0
ENV PORT=8000

COPY --from=builder /app/target/release/rust-backend /usr/local/bin/rust-backend

EXPOSE 8000
HEALTHCHECK --interval=30s --timeout=5s --retries=5 CMD \
    /bin/sh -c "wget -qO- http://127.0.0.1:${PORT}/healthz || exit 1"

CMD ["rust-backend"]