# syntax=docker/dockerfile:1

# Build stage
FROM rust:1.80-slim AS builder
WORKDIR /app

# System deps for reqwest native-tls
RUN apt-get update && \
    apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates build-essential && \
    rm -rf /var/lib/apt/lists/*

# Cache deps
COPY rust-backend/Cargo.toml rust-backend/Cargo.lock ./
# Create a dummy src to build dep graph
RUN mkdir -p src && echo "fn main(){}" > src/main.rs && \
    cargo build --release && rm -rf target/release/deps/rust_backend*

# Copy actual source
COPY rust-backend/ ./

# Build release
RUN cargo build --release

# Runtime stage
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


