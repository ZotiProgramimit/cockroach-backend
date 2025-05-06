    FROM rust:1.83-slim-bookworm AS builder
    WORKDIR /app
    
    RUN apt-get update \
     && apt-get install -y --no-install-recommends protobuf-compiler \
     && rm -rf /var/lib/apt/lists/*
    
    COPY . .
    RUN cargo build --release --locked -j 1
    
    FROM debian:bookworm-slim
    ENV DEBIAN_FRONTEND=noninteractive
    RUN apt-get update \
     && apt-get install -y --no-install-recommends ca-certificates \
     && rm -rf /var/lib/apt/lists/*
    COPY --from=builder /app/target/release/casino-backend /usr/local/bin/
    EXPOSE 50051 3000
    CMD ["casino-backend"]
    