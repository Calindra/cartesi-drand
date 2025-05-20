# syntax=docker.io/docker/dockerfile:1

# This enforces that the packages downloaded from the repositories are the same
# for the defined date, no matter when the image is built.
ARG UBUNTU_TAG=noble-20250404
ARG APT_UPDATE_SNAPSHOT=20250424T030400Z

################################################################################
# riscv64 base stage
FROM --platform=linux/riscv64 ubuntu:${UBUNTU_TAG} AS base-riscv64

ARG APT_UPDATE_SNAPSHOT
ARG DEBIAN_FRONTEND=noninteractive
RUN <<EOF
set -eu
apt-get update
apt-get install -y --no-install-recommends ca-certificates curl
apt-get update --snapshot=${APT_UPDATE_SNAPSHOT}
EOF

################################################################################
# cross base stage
FROM --platform=$BUILDPLATFORM ubuntu:${UBUNTU_TAG} AS base-cross

ARG APT_UPDATE_SNAPSHOT
ARG DEBIAN_FRONTEND=noninteractive
RUN <<EOF
set -eu
apt-get update
apt-get install -y --no-install-recommends ca-certificates curl
apt-get update --snapshot=${APT_UPDATE_SNAPSHOT}
EOF

################################################################################
# cross build stage
FROM base-cross AS cross-build-stage

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.82.0

ARG DEBIAN_FRONTEND=noninteractive
RUN <<EOF
set -e
apt-get install -y --no-install-recommends \
    build-essential \
    g++-riscv64-linux-gnu
EOF

RUN <<EOF
set -eux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --target riscv64gc-unknown-linux-gnu
chmod -R a+w $RUSTUP_HOME $CARGO_HOME
rustup --version
cargo --version
rustc --version
EOF

WORKDIR /opt/cartesi/dapp

FROM cross-build-stage as dapp-contract

COPY dapp-contract-blackjack dapp-contract-blackjack
COPY dapp-contract-blackjack/.cargo/config.docker.toml dapp-contract-blackjack/.cargo/config.toml

WORKDIR /opt/cartesi/dapp/dapp-contract-blackjack

RUN cargo build --release --target riscv64gc-unknown-linux-gnu

FROM cross-build-stage as middleware

COPY convenience-middleware convenience-middleware
COPY convenience-middleware/.cargo/config.docker.toml convenience-middleware/.cargo/config.toml

WORKDIR /opt/cartesi/dapp/convenience-middleware

RUN cargo build --release --target riscv64gc-unknown-linux-gnu

################################################################################
# runtime stage: produces final image that will be executed
FROM base-riscv64

ARG MACHINE_GUEST_TOOLS_VERSION=0.17.0
ARG DEBIAN_FRONTEND=noninteractive
RUN <<EOF
set -e
apt-get install -y --no-install-recommends \
    busybox-static

cd /tmp
busybox wget https://github.com/cartesi/machine-guest-tools/releases/download/v${MACHINE_GUEST_TOOLS_VERSION}/machine-guest-tools_riscv64.deb
echo "973943b3a3e40164175da7d7b5b7857642d1277e1fd38be268da12daca5ff458735f93a7ac25b350b3de58b073a25b64c860d9eb92157bfc946b03dd1a345cc9 /tmp/machine-guest-tools_riscv64.deb" \
    | sha512sum -c
apt-get install -y --no-install-recommends \
    /tmp/machine-guest-tools_riscv64.deb
rm /tmp/machine-guest-tools_riscv64.deb

rm -rf /var/lib/apt/lists/* /var/log/* /var/cache/*
EOF

ENV PATH="/opt/cartesi/bin:/opt/cartesi/dapp:${PATH}"

WORKDIR /opt/cartesi/dapp

COPY --from=dapp-contract --chmod=755 /opt/cartesi/dapp/dapp-contract-blackjack/target/riscv64gc-unknown-linux-gnu/release/dapp-contract-blackjack .
COPY --from=middleware --chmod=755 /opt/cartesi/dapp/convenience-middleware/target/riscv64gc-unknown-linux-gnu/release/cartesi-drand .
COPY --chmod=644 convenience-middleware/drand.config.json convenience-middleware/.env ./
COPY --chmod=755 dapp-start.sh ./

ENV ROLLUP_HTTP_SERVER_URL="http://127.0.0.1:5004"

ENTRYPOINT ["rollup-init"]
CMD ["dapp"]
