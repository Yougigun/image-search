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

FROM rust:1.82.0 AS web-server
COPY --from=builder /usr/src/app/target/release/web-server /usr/local/bin/web-server
COPY --from=builder /usr/src/app/services/web-server/config.yaml /config.yaml
EXPOSE 3000
ENTRYPOINT ["web-server"]
LABEL service=web-server