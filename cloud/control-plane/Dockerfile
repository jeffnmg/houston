FROM rust:1-slim-bookworm AS build
WORKDIR /src
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/houston-control-plane /usr/local/bin/houston-control-plane
ENV HOUSTON_CP_BIND=0.0.0.0:8080 \
    HOUSTON_CP_DB_PATH=/data/control-plane.db \
    RUST_LOG=info,houston_control_plane=debug
EXPOSE 8080
CMD ["/usr/local/bin/houston-control-plane"]
