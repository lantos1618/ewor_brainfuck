FROM mcr.microsoft.com/devcontainers/rust:1-bullseye

# Install additional OS packages
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
        pkg-config \
        libssl-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Install additional Rust components
RUN rustup component add clippy rustfmt 