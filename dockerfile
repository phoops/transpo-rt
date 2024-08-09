FROM rust:bookworm AS builder

WORKDIR /usr/src/app

COPY . .

ENV OPENSSL_DIR=/usr/lib/ssl
ENV OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu
ENV OPENSSL_INCLUDE_DIR=/usr/include/openssl

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y openssl build-essential ca-certificates

WORKDIR /usr/src/app

COPY --from=builder /usr/src/app/target/release/transpo-rt .

CMD ["./transpo-rt", "-c", "/usr/src/app/config.yml"]