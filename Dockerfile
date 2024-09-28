FROM debian:bookworm-slim

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY target/release/pv-inv-bridge /app/pv-inv-bridge

ENTRYPOINT ["/app/pv-inv-bridge"]
