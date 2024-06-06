# syntax=docker.io/docker/dockerfile:1.4
ARG FOLDER_MIDDLEWARE=convenience-middleware

FROM rust:1.78.0-bookworm as middleware

ARG FOLDER_MIDDLEWARE
ARG DEBIAN_FRONTEND=noninteractive

# https://github.com/moby/buildkit/blob/master/frontend/dockerfile/docs/reference.md#example-cache-apt-packages
RUN <<EOF
rm -f /etc/apt/apt.conf.d/docker-clean
echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
EOF

RUN \
  --mount=type=cache,target=/var/cache/apt,sharing=locked \
  --mount=type=cache,target=/var/lib/apt,sharing=locked \
  apt-get update && \
  apt-get install -y --no-install-recommends \
  # build-essential=12.9 \
  # ca-certificates=20230311 \
  g++-riscv64-linux-gnu=4:12.2.0-5

RUN rustup target add riscv64gc-unknown-linux-gnu

WORKDIR /usr/src/${FOLDER_MIDDLEWARE}

COPY convenience-middleware .

# https://docs.docker.com/build/cache/#use-the-dedicated-run-cache
# https://docs.docker.com/engine/reference/builder/#run---mounttypecache
RUN \
  --mount=type=cache,target=/usr/local/cargo/registry/,sharing=locked \
  cargo build --release --target=riscv64gc-unknown-linux-gnu

FROM rust:1.78.0-bookworm as dapp-contract

ARG DEBIAN_FRONTEND=noninteractive
RUN <<EOF
rm -f /etc/apt/apt.conf.d/docker-clean
echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
EOF

RUN \
  --mount=type=cache,target=/var/cache/apt,sharing=locked \
  --mount=type=cache,target=/var/lib/apt,sharing=locked \
  apt-get update && \
  apt-get install -y --no-install-recommends \
  # build-essential=12.9 \
  # ca-certificates=20230311 \
  g++-riscv64-linux-gnu=4:12.2.0-5

RUN rustup target add riscv64gc-unknown-linux-gnu

WORKDIR /opt/cartesi/dapp
COPY dapp-contract-blackjack .
RUN \
  --mount=type=cache,target=/usr/local/cargo/registry/,sharing=locked \
  cargo build --release

FROM --platform=linux/riscv64 riscv64/ubuntu:22.04

ARG FOLDER_MIDDLEWARE

# Flags: https://github.com/cartesi/cli/blob/65fb9fd557f93d6624cf86a7b9b3d3f8277423e0/apps/cli/src/commands/build.ts#L26-L33
LABEL io.cartesi.sdk_version=0.6.2
LABEL io.cartesi.rollups.ram_size=128Mi

ARG DEBIAN_FRONTEND=noninteractive
# Releases: https://github.com/cartesi/machine-emulator-tools/releases
ARG MACHINE_EMULATOR_TOOLS_VERSION=0.15.0

RUN <<EOF
apt-get update
apt-get install -y --no-install-recommends \
    busybox-static=1:1.30.1-7ubuntu3 \
    ca-certificates=20230311ubuntu0.22.04.1 \
    curl=7.81.0-1ubuntu1.15 \
    vim=2:8.2.3995-1ubuntu2.15 \
    jq=1.6-2.1ubuntu3
curl -fsSL https://github.com/cartesi/machine-emulator-tools/releases/download/v${MACHINE_EMULATOR_TOOLS_VERSION}/machine-emulator-tools-v${MACHINE_EMULATOR_TOOLS_VERSION}.tar.gz \
  | tar -C / --overwrite -xvzf -
rm -rf /var/lib/apt/lists/*
EOF

ARG CARTESI_DRAND_VERSION=0.2.10

ENV PATH="/opt/cartesi/bin:/opt/cartesi/dapp:${PATH}"

WORKDIR /opt/cartesi/dapp
COPY --from=middleware /usr/src/${FOLDER_MIDDLEWARE}/target/riscv64gc-unknown-linux-gnu/release/cartesi-drand .
COPY --from=dapp-contract /opt/cartesi/dapp/target/riscv64gc-unknown-linux-gnu/release/dapp-contract-blackjack .
COPY dapp-start.sh ${FOLDER_MIDDLEWARE}/drand.config.json ${FOLDER_MIDDLEWARE}/.env ./

ENV ROLLUP_HTTP_SERVER_URL="http://127.0.0.1:5004"

RUN chmod +x dapp-start.sh cartesi-drand

ENTRYPOINT ["rollup-init"]
CMD ["dapp-start.sh"]
