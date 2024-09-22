FROM debian:bookworm-slim as main-linux-arm64
COPY target/aarch64-unknown-linux-gnu/release/pv-inv-bridge /usr/local/bin/pv-inv-bridge

FROM debian:bookworm-slim as main-linux-amd64
COPY target/release/pv-inv-bridge /usr/local/bin/pv-inv-bridge

FROM main-${TARGETOS}-${TARGETARCH}${TARGETVARIANT}
CMD ["pv-inv-bridge"]
