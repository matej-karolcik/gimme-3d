FROM rust:1.74.1-slim-bookworm as builder

RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    libfontconfig1-dev \
    xvfb \
    libxcursor-dev \
    libxi-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/render
COPY . .

RUN cargo build --release --bin render

FROM alpine:3.19.0

COPY --from=builder /usr/src/render/target/release/render /usr/local/bin/render
