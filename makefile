shell:
	docker run --platform linux/amd64 -it --rm -v $(PWD):/app -w /app rust:1.74.1-slim-bookworm bash

shell-2stage:
	docker run -it --rm -w /app -v $$(pwd)/results:/app/results 3d-renderer-2stage bash

install:
	apt-get update && apt-get install -y libfontconfig1-dev build-essential \
libxcursor-dev libxi-dev libxrandr-dev libx11-dev libx11-xcb1

rust:
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

run:
	xvfb-run cargo run --release

build:
	docker build -t 3d-renderer .

build-2stage:
	docker build -t 3d-renderer-2stage -f Dockerfile.2stage .
