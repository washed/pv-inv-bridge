FROM rust as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .

FROM rust
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/app
COPY --from=builder /usr/local/cargo/bin/pv-inv-bridge /usr/src/app
CMD ["/usr/src/app/pv-inv-bridge"]
