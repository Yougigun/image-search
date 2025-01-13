# syntax=docker/dockerfile:1

FROM rust:1.82 AS sqlx-builder
WORKDIR /usr/src/sqlx-cli
RUN cargo install --version 0.8.2 sqlx-cli

FROM alpine:3.20.3 AS urlencode-builder
COPY <<EOF /usr/local/bin/urlencode
#!/bin/python3
import sys, urllib.parse
print(urllib.parse.quote(sys.argv[1]))
EOF
RUN chmod +x /usr/local/bin/urlencode

FROM golang:1.22 AS k6-builder
WORKDIR /usr/src/k6
RUN go install go.k6.io/xk6/cmd/xk6@latest && \
        xk6 build \
        --with github.com/grafana/xk6-sql \
        --with github.com/grafana/xk6-sql-driver-postgres

FROM debian:trixie

# install python to execute `urlencode`
RUN apt-get update && \
    apt-get install -y python3 python3-pip && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

COPY --from=sqlx-builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY --from=urlencode-builder /usr/local/bin/urlencode /usr/local/bin/urlencode
COPY --from=k6-builder /usr/src/k6/k6 /usr/local/bin/k6
