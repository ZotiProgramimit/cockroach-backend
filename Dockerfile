# ---------- build ----------
    FROM rust:1.83-slim-bookworm AS builder     # ← newer Cargo
    WORKDIR /app
    COPY . .
    # -j 1 keeps memory low on Railway’s 1 GiB build VMs
    RUN cargo build --release --locked -j 1
    
    # ---------- runtime ----------
    FROM debian:bookworm-slim
    RUN apt-get update          \
     && apt-get install -y --no-install-recommends ca-certificates \
     && rm -rf /var/lib/apt/lists/*
    COPY --from=builder /app/target/release/casino-backend /usr/local/bin/
    EXPOSE 50051 3000
    CMD ["casino-backend"]
    