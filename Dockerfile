FROM debian:bookworm-slim as main-linux-armv7
# do whatever is different on linux/arm/v7
COPY target/aarch64-unknown-linux-gnu/release/pv-inv-bridge /usr/local/bin/pv-inv-bridge

FROM debian:bookworm-slim as main-linux-amd64
# do whatever should be for linux-amd64
COPY target/x86_64-unknown-linux-gnu/release/pv-inv-bridge /usr/local/bin/pv-inv-bridge

FROM main-linux-amd64 as main-linux-arm64
# linux-arm64 is the same as linux-amd64 but every target needs to be defined

FROM main-${TARGETOS}-${TARGETARCH}${TARGETVARIANT}
CMD ["pv-inv-bridge"]
