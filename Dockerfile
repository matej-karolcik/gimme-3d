FROM rust:1.74.1-slim-bookworm

RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    libfontconfig1-dev \
    xvfb \
    libxcursor-dev \
    libxi-dev \
    && rm -rf /var/lib/apt/lists/*

RUN cargo build --release --bin render
