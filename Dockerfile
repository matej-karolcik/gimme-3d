# syntax = docker/dockerfile:1.6

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
    cargo build --release --bin cmd \
    && mv target/release/cmd /tmp/cmd

FROM debian:trixie-20231218-slim

RUN apt-get update && apt-get install -y \
    libfontconfig1-dev \
    xvfb \
    libxcursor-dev \
    libssl-dev \
    ca-certificates \
    libxi-dev \
    && rm -rf /var/lib/apt/lists/*

# todo remove this
RUN apt-get update && apt-get install -y \
    curl \
    procps

COPY --from=builder /tmp/cmd /app/cmd

WORKDIR /app

# todo remove this
COPY testdata/canvas.png testdata/canvas.png

ENV TINI_VERSION v0.19.0
ADD https://github.com/krallin/tini/releases/download/${TINI_VERSION}/tini /tini
RUN chmod +x /tini

ENTRYPOINT ["/tini", "--"]
CMD ["xvfb-run", "-a", "./cmd", "serve"]
