ARG TARGETOS
ARG TARGETARCH

FROM --platform=$BUILDPLATFORM rust:1-alpine AS builder
ARG TARGETPLATFORM
RUN echo $TARGETPLATFORM

RUN apk add --no-cache musl-dev

# dummy project to cache deps
WORKDIR /usr/src
RUN cargo new pv-inv-bridge
COPY Cargo.toml Cargo.lock /usr/src/pv-inv-bridge/
WORKDIR /usr/src/pv-inv-bridge
RUN cargo build --release

# build with actual source
COPY src/ /usr/src/pv-inv-bridge/src/
RUN touch /usr/src/pv-inv-bridge/src/main.rs
RUN cargo build --release

FROM rust:1-alpine
COPY --from=builder /usr/src/pv-inv-bridge/target/release/pv-inv-bridge /usr/local/bin
CMD ["pv-inv-bridge"]
