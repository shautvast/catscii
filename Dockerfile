FROM ubuntu:22.04

SHELL ["/bin/bash", "-c"]

RUN set -eux; \
    apt update; \
    apt install -y curl ca-certificates gcc libssl-dev build-essential pkg-config;

RUN set -eux; \
    curl --location --fail \
      "https://static.rust-lang.org/rustup/dist/aarch64-unknown-linux-gnu/rustup-init" \
      --output rustup-init; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --default-toolchain stable; \
    rm rustup-init;

ENV PATH=${PATH}:/root/.cargo/bin
WORKDIR /app
COPY src src
COPY Cargo.toml Cargo.lock ./
RUN set -eux; \
    cargo build --release; \
    cp target/release/catscii .; \
    rm -rf /var/lib/{apt,dpkg,cache,log}/

CMD ["/app/catscii"]
