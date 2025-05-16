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

# https://docs.docker.com/build/cache/#use-the-dedicated-run-cache
# https://docs.docker.com/engine/reference/builder/#run---mounttypecache
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

FROM --platform=linux/riscv64 riscv64/ubuntu:24.04

RUN useradd --create-home --user-group dapp

# Releases: https://github.com/cartesi/machine-guest-tools/releases
ARG MACHINE_EMULATOR_TOOLS_VERSION=0.17.0
ADD --checksum=sha256:ee205c345818c682fb1dfedd3fe3e4a074148e643ee4b3abad9cefd747877177 https://github.com/cartesi/machine-guest-tools/releases/download/v${MACHINE_EMULATOR_TOOLS_VERSION}/machine-guest-tools_riscv64.deb /tmp/machine-guest-tools_riscv64.deb
RUN <<EOF
    dpkg -i /tmp/machine-guest-tools_riscv64.deb
    rm /tmp/machine-guest-tools_riscv64.deb
EOF

ENV DEBIAN_FRONTEND=noninteractive
RUN <<EOF
set -e
apt-get update
apt-get install -y --no-install-recommends \
    busybox-static \
    jq libjq1 libonig5
EOF

USER dapp

# Flags: https://github.com/cartesi/cli/blob/65fb9fd557f93d6624cf86a7b9b3d3f8277423e0/apps/cli/src/commands/build.ts#L26-L33
LABEL io.cartesi.rollups.sdk_version=0.11.1
LABEL io.cartesi.rollups.ram_size=128Mi

ENV PATH="/opt/cartesi/bin:/opt/cartesi/dapp:${PATH}"

WORKDIR /opt/cartesi/dapp
COPY --from=dapp-contract --chown=dapp:dapp --chmod=755 /opt/cartesi/dapp/dapp-contract-blackjack/target/riscv64gc-unknown-linux-gnu/release/dapp-contract-blackjack .
COPY --from=middleware --chown=dapp:dapp --chmod=755 /opt/cartesi/dapp/convenience-middleware/target/riscv64gc-unknown-linux-gnu/release/cartesi-drand .
# COPY --chown=dapp:dapp ./convenience-middleware/drand.config.json ./convenience-middleware/
COPY --chown=dapp:dapp --chmod=644 convenience-middleware/drand.config.json convenience-middleware/.env ./
COPY --chown=dapp:dapp --chmod=755 dapp-start.sh ./

ENV ROLLUP_HTTP_SERVER_URL="http://127.0.0.1:5004"

RUN mkdir -pv data/address data/names

ENTRYPOINT ["rollup-init"]
CMD ["dapp-start.sh"]