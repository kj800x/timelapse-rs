FROM rust:1.91

RUN apt-get update && apt-get install -y --no-install-recommends ffmpeg && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/timelapse-rs
COPY . .

RUN cargo install --path .

CMD ["timelapse-rs"]
