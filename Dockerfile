FROM rust:1.70.0 as builder
WORKDIR /usr/src/gpx-track-bot
COPY . .
RUN cargo install --path .
FROM debian:12-slim
# RUN apt-get update & apt-get install -y extra-runtime-dependencies & rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/gpx-track-bot /usr/local/bin/gpx-track-bot
CMD ["gpx-track-bot"]