# syntax=docker.io/docker/dockerfile:1.4
ARG FOLDER=convenience-middleware

FROM rust:1.73.0-bookworm as builder

ARG FOLDER

WORKDIR /usr/src/${FOLDER}

COPY . .

RUN cargo build --release

FROM ubuntu:22.04

ARG FOLDER

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    ca-certificates=20230311ubuntu0.22.04.1 \
    busybox-static=1:1.30.1-7ubuntu3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/${FOLDER}/target/release/cartesi-drand .
COPY --from=builder /usr/src/${FOLDER}/drand.config.json .

EXPOSE 8080

CMD ["/usr/local/bin/cartesi-drand"]