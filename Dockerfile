# ---------- build ----------
    FROM rust:1.77 AS builder
    WORKDIR /app
    COPY . .
    RUN --mount=type=cache,target=/usr/local/cargo/registry \
        --mount=type=cache,target=/usr/local/cargo/git \
        cargo build --release
    
    # ---------- runtime ----------
    FROM debian:bookworm-slim
    RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
    COPY --from=builder /app/target/release/casino-backend /usr/local/bin/
    EXPOSE 50051 3000
    CMD ["casino-backend"]
    