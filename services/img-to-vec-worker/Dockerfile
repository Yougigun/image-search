FROM rust:1.82.0 AS chef
RUN cargo install cargo-chef
WORKDIR /usr/src/app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /usr/src/app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM rust:1.82.0 AS img-to-vec-worker
COPY --from=builder /usr/src/app/target/release/img-to-vec-worker /usr/local/bin/img-to-vec-worker
COPY --from=builder /usr/src/app/services/img-to-vec-worker/config.yaml /config.yaml
EXPOSE 3000
ENTRYPOINT ["img-to-vec-worker"]
LABEL service=img-to-vec-worker
