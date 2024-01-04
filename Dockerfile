# syntax = docker/dockerfile:1.2

FROM rust:1.74.1-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    libfontconfig1-dev \
    xvfb \
    libxcursor-dev \
    libssl-dev \
    libxi-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/renderer

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/renderer/target \
    cargo build --release --bin server \
    && mv target/release/server /tmp/server

FROM debian:trixie-20231218-slim

RUN apt-get update && apt-get install -y \
    libfontconfig1-dev \
    xvfb \
    libxcursor-dev \
    libssl-dev \
    ca-certificates \
    libxi-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /tmp/server /app/server

ENTRYPOINT ["xvfb-run", "-a", "/app/server"]
