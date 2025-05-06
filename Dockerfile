# syntax=docker/dockerfile:1

########################
#  Stage 1 – builder   #
########################
FROM rust:slim AS builder

# Needed for tonic-build (protoc) + SQLx TLS
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      protobuf-compiler pkg-config libssl-dev ca-certificates \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# --- 1. cache dependencies ---
COPY Cargo.toml Cargo.lock build.rs ./
COPY proto ./proto
# dummy src so `cargo build` grabs deps only
RUN mkdir -p src && echo "fn main(){}" > src/main.rs
RUN RUSTFLAGS="-Ctarget-cpu=native" cargo build --release

# --- 2. real build ---
COPY src ./src
RUN cargo build --release

########################
#  Stage 2 – runtime   #
########################
FROM debian:bookworm-slim

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/casino-backend /usr/local/bin/
# Optional: hold default .env for local dev – prod values come from compose/railway vars
COPY .env .env

ENV RUST_LOG=info
EXPOSE 3000 50051
CMD ["casino-backend"]
