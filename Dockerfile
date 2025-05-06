# --- build stage -------------------------------------------------
    FROM rust:1.77 as builder
    WORKDIR /app
    COPY . .
    RUN --mount=type=cache,target=/usr/local/cargo/registry \
        --mount=type=cache,target=/usr/local/cargo/git \
        cargo build --release
    
    # --- runtime stage ----------------------------------------------
    FROM debian:bookworm-slim
    RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
    WORKDIR /app
    COPY --from=builder /app/target/release/casino-backend /usr/local/bin/casino-backend
    EXPOSE 3000 50051
    CMD ["casino-backend"]
    