# Get started with a build env with Rust nightly
# FROM rustlang/rust:nightly-bookworm as builder

ARG RUST_VERSION=1.96.0
ARG APP_NAME=gmr

# If you’re using stable, use this instead
FROM rust:${RUST_VERSION}-bookworm AS builder
ARG APP_NAME
ENV LEPTOS_WASM_BINDGEN_VERSION=0.2.125

# Install cargo-binstall, which makes it easier to install other
# cargo extensions like cargo-leptos
RUN wget https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz
RUN tar -xvf cargo-binstall-x86_64-unknown-linux-musl.tgz
RUN cp cargo-binstall /usr/local/cargo/bin

# Install required tools
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends clang

# Install cargo-leptos
RUN cargo binstall cargo-leptos --version 0.3.6 -y

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown


# Make an /app dir, which everything will eventually live in
RUN mkdir -p /app
WORKDIR /app

RUN cargo install wasm-bindgen-cli --version 0.2.125 --force

COPY src src/
COPY style style/
COPY public public/
COPY locales locales/
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
# Build the app
##RUN cargo leptos build --release -vv
RUN --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    rm -rf /app/target/site && \
    cargo leptos build --release -vv && \
    cp /app/target/release/${APP_NAME} /bin/server && \
    cp -r /app/target/site/ /bin/site

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*

RUN update-ca-certificates

# Copy the server binary to the /app directory
COPY --from=builder /bin/server /app/
# /target/site contains our JS/WASM/CSS, etc.
COPY --from=builder /bin/site /app/site
# Copy Cargo.toml if it’s needed at runtime
# COPY --from=builder /app/Cargo.toml /app/

# Set any required env variables and
ENV RUST_LOG="info"
ENV APP_ENV=PROD
ENV RUST_BACKTRACE=full
ENV LEPTOS_WASM_BINDGEN_VERSION=0.2.125
ENV LEPTOS_SITE_ADDR="0.0.0.0:3080"
ENV LEPTOS_SITE_ROOT="site"
EXPOSE 3080

# -- NB: update binary name from "leptos_start" to match your app name in Cargo.toml --
# Run the server
CMD ["/app/server"]
