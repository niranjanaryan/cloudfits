# cloudfits — multi-stage build
FROM rust:1.85 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin cloudfits

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/cloudfits /usr/local/bin/cloudfits
ENTRYPOINT ["cloudfits"]
CMD ["--help"]
