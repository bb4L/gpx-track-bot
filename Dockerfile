FROM rust:1.70.0 as builder
WORKDIR /usr/src/gpx-track-bot
COPY . .
RUN cargo install --path .
FROM debian:12-slim
WORKDIR /app
RUN apt-get update -y && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/gpx-track-bot/target/release/gpx-track-bot /app/gpx-track-bot
CMD ["/app/gpx-track-bot"]