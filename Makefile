.PHONY: build

build:
	-docker buildx create --use --platform linux/amd64,linux/arm64
	docker buildx build --platform linux/amd64,linux/arm64 -t pv-inv-bridge:latest .
	docker buildx build -t pv-inv-bridge:latest --load --progress plain .

debug-build:
	-docker buildx create --use --name larger_log_2 --platform linux/arm64 --driver-opt env.BUILDKIT_STEP_LOG_MAX_SIZE=50000000
	docker buildx build --platform linux/arm64 -t pv-inv-bridge:latest --load --progress plain .
