FROM rust:1.86-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY crates ./crates
COPY config ./config

RUN apt-get update && apt-get install -y \
    pkg-config \
    libx11-dev \
    libxtst-dev \
    libxi-dev \
    libxdo-dev \
 && rm -rf /var/lib/apt/lists/*

RUN cargo build --release -p xbuttonmousecontrol-cli

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libx11-6 \
    libxtst6 \
    libxi6 \
    libxdo3 \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/xbuttonmousecontrol-cli /usr/local/bin/xbuttonmousecontrol-cli
COPY config/bindings.toml /app/config/bindings.toml

CMD ["xbuttonmousecontrol-cli", "/app/config/bindings.toml"]