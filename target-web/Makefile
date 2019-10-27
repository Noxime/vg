CMD=docker run --rm \
	-v "$$(pwd)/:/root/vg/src" \
	-v "$$(pwd)/../../:/root/" \
	-v "$$(pwd)/../engine:/root/vg/engine" \
	-v "$$(pwd)/../util/:/root/vg/util" \
	-w "/root/vg/src" \
	vg-wasm

build:
	$(CMD) cargo web deploy

x:
	$(CMD) nodejs --version

docker:
	docker build . -t vg-wasm