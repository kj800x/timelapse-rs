# Build Stage
FROM rust:1.96 AS builder

WORKDIR /usr/src
RUN USER=root cargo new --vcs none timelapse-rs
WORKDIR /usr/src/timelapse-rs

# Pre-cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

# Copy actual source and rebuild
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Runtime Stage
FROM debian:trixie-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates libssl3 ffmpeg \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/timelapse-rs/target/release/timelapse-rs /usr/local/bin/timelapse-rs

CMD ["timelapse-rs"]
