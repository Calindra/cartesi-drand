# syntax=docker.io/docker/dockerfile:1
FROM rust:1.86.0-bookworm AS builder

# Working with @cartesi/cli --version
# @cartesi/cli/0.14.1 darwin-arm64 node-v20.10.0

ENV DEBIAN_FRONTEND=noninteractive

# https://github.com/moby/buildkit/blob/master/frontend/dockerfile/docs/reference.md#example-cache-apt-packages
RUN <<EOF
rm -f /etc/apt/apt.conf.d/docker-clean
echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
EOF

RUN \
  --mount=type=cache,target=/var/cache/apt,sharing=locked \
  --mount=type=cache,target=/var/lib/apt,sharing=locked \
<<EOF
    set -e
    apt-get update
    apt-get install -y --no-install-recommends \
        g++-riscv64-linux-gnu
EOF

RUN rustup target add riscv64gc-unknown-linux-gnu

WORKDIR /opt/cartesi/dapp

FROM builder AS dapp-contract

COPY dapp-contract-blackjack dapp-contract-blackjack
COPY dapp-contract-blackjack/.cargo/config.docker.toml dapp-contract-blackjack/.cargo/config.toml

WORKDIR /opt/cartesi/dapp/dapp-contract-blackjack

RUN \
  --mount=type=cache,target=/usr/local/cargo/registry/,sharing=locked \
    cargo build --release --target riscv64gc-unknown-linux-gnu

FROM builder AS middleware

COPY convenience-middleware convenience-middleware
COPY convenience-middleware/.cargo/config.docker.toml convenience-middleware/.cargo/config.toml

WORKDIR /opt/cartesi/dapp/convenience-middleware

RUN \
  --mount=type=cache,target=/usr/local/cargo/registry/,sharing=locked \
  cargo build --release --target riscv64gc-unknown-linux-gnu

# COPY Cargo.toml .
# RUN cargo build --release --workspace --target riscv64gc-unknown-linux-gnu

FROM --platform=linux/riscv64 riscv64/ubuntu:24.04 AS final

RUN useradd --create-home --user-group dapp

# Releases: https://github.com/cartesi/machine-guest-tools/releases
ARG FOLDER_MIDDLEWARE=convenience-middleware
ARG MACHINE_EMULATOR_TOOLS_VERSION=0.17.0
ADD https://github.com/cartesi/machine-guest-tools/releases/download/v${MACHINE_EMULATOR_TOOLS_VERSION}/machine-guest-tools_riscv64.deb /tmp/machine-guest-tools_riscv64.deb
RUN <<EOF
    dpkg -i /tmp/machine-guest-tools_riscv64.deb
    rm /tmp/machine-guest-tools_riscv64.deb
EOF


# Flags: https://github.com/cartesi/cli/blob/65fb9fd557f93d6624cf86a7b9b3d3f8277423e0/apps/cli/src/commands/build.ts#L26-L33
# LABEL io.cartesi.rollups.sdk_version=0.6.2
LABEL io.cartesi.rollups.ram_size=128Mi

ENV DEBIAN_FRONTEND=noninteractive
RUN <<EOF
set -e
apt-get update
apt-get install -y --no-install-recommends \
    busybox-static
EOF

ENV PATH="/opt/cartesi/bin:/opt/cartesi/dapp:${PATH}"

WORKDIR /opt/cartesi/dapp
COPY --from=dapp-contract /opt/cartesi/dapp/dapp-contract-blackjack/target/riscv64gc-unknown-linux-gnu/release/dapp-contract-blackjack .
COPY --from=middleware /opt/cartesi/dapp/convenience-middleware/target/riscv64gc-unknown-linux-gnu/release/cartesi-drand .
COPY convenience-middleware/drand.config.json ./convenience-middleware/
COPY dapp-start.sh convenience-middleware/drand.config.json convenience-middleware/.env ./

ENV ROLLUP_HTTP_SERVER_URL="http://127.0.0.1:5004"

RUN chmod +x dapp-start.sh cartesi-drand dapp-contract-blackjack
RUN mkdir -p data/address data/names

ENTRYPOINT ["rollup-init"]
CMD ["dapp-start.sh"]