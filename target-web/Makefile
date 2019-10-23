CMD=docker run --rm \
	-v "$$(pwd)/:/root/kea/src" \
	-v "$$(pwd)/../../:/root/" \
	-v "$$(pwd)/../engine:/root/kea/engine" \
	-v "$$(pwd)/../util/:/root/kea/util" \
	-w "/root/kea/src" \
	kea-wasm

build:
	$(CMD) cargo web deploy

x:
	$(CMD) nodejs --version

docker:
	docker build . -t kea-wasm