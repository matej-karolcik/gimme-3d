shell:
	docker run --platform linux/amd64 -it --rm -v $(PWD):/app -w /app rust:1.74.1-slim-bookworm bash

# todo try amd64 to build
install:
	apt-get update && apt-get install -y libfontconfig1-dev build-essential \
libxcursor-dev libxi-dev libxrandr-dev libx11-dev libx11-xcb1
rust:
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
run:
	xvfb-run cargo run --release
