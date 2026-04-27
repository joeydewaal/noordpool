# syntax=docker/dockerfile:1.7

FROM rust:1.95-slim-bookworm AS build
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
RUN cargo build --release --features prod

FROM debian:stable-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /app/target/release/noordpool-backend /app/noordpool-backend
ENV PORT=3000
EXPOSE 3000
CMD ["/app/noordpool-backend"]
