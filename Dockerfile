# syntax=docker.io/docker/dockerfile:1
FROM rust:1.86.0-bookworm AS builder

# Working with @cartesi/cli --version
# @cartesi/cli/0.14.1 darwin-arm64 node-v20.10.0

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.86.0

ENV DEBIAN_FRONTEND=noninteractive

# RUN <<EOF
# set -e
# apt update
# apt install -y --no-install-recommends \
#     build-essential=12.9ubuntu3 \
#     ca-certificates=20230311ubuntu0.22.04.1 \
#     g++-riscv64-linux-gnu=4:11.2.0--1ubuntu1 \
#     wget=1.21.2-2ubuntu1
# EOF

RUN rustup target add riscv64gc-unknown-linux-gnu

WORKDIR /opt/cartesi/dapp

COPY dapp-contract-blackjack dapp-contract-blackjack
COPY dapp-contract-blackjack/.cargo/config.docker.toml dapp-contract-blackjack/.cargo/config.toml
WORKDIR /opt/cartesi/dapp/dapp-contract-blackjack
RUN cargo build --release

WORKDIR /opt/cartesi/dapp

COPY convenience-middleware convenience-middleware
COPY convenience-middleware/.cargo/config.docker.toml convenience-middleware/.cargo/config.toml
WORKDIR /opt/cartesi/dapp/convenience-middleware
RUN cargo build --release

WORKDIR /opt/cartesi/dapp

# COPY Cargo.toml .
# RUN cargo build --release --workspace --target=riscv64gc-unknown-linux-gnu

FROM --platform=linux/riscv64 riscv64/ubuntu:24.04

ARG FOLDER_MIDDLEWARE=convenience-middleware
ARG MACHINE_EMULATOR_TOOLS_VERSION=0.17.0
ADD https://github.com/cartesi/machine-guest-tools/releases/download/v${MACHINE_EMULATOR_TOOLS_VERSION}/machine-guest-tools_riscv64.deb /
RUN apt-get install -i /machine-guest-tools_riscv64.deb \
    && rm /machine-guest-tools_riscv64.deb

# LABEL io.cartesi.rollups.sdk_version=0.6.2
LABEL io.cartesi.rollups.ram_size=128Mi

ENV DEBIAN_FRONTEND=noninteractive
RUN <<EOF
set -e
apt-get update
apt-get install -y --no-install-recommends \
    busybox-static=1:1.36.1-6ubuntu3 \
    jq=1.7.1-3build1
rm -rf /var/lib/apt/lists/* /var/log/* /var/cache/*
useradd --create-home --user-group dapp
EOF

ENV PATH="/opt/cartesi/bin:/opt/cartesi/dapp:${PATH}"

WORKDIR /opt/cartesi/dapp
COPY --from=builder /opt/cartesi/dapp/dapp-contract-blackjack/target/riscv64gc-unknown-linux-gnu/release/dapp-contract-blackjack .
COPY --from=builder /opt/cartesi/dapp/convenience-middleware/target/riscv64gc-unknown-linux-gnu/release/cartesi-drand .
COPY convenience-middleware/drand.config.json ./convenience-middleware/
COPY dapp-start.sh convenience-middleware/drand.config.json convenience-middleware/.env ./

ENV ROLLUP_HTTP_SERVER_URL="http://127.0.0.1:5004"

RUN chmod +x dapp-start.sh cartesi-drand dapp-contract-blackjack
RUN mkdir -p data/address data/names

ENTRYPOINT ["rollup-init"]
CMD ["dapp-start.sh"]