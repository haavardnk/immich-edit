FROM node:22-slim AS web
WORKDIR /build
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web/ .
RUN npm run build

FROM rust:1.89-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    nasm cmake pkg-config libclang-dev && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /build
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/
RUN cargo build --release --bin immich-edit -j$(nproc) && \
    strip target/release/immich-edit

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libturbojpeg0 \
    libvulkan1 \
    mesa-vulkan-drivers && \
    rm -rf /var/lib/apt/lists/*
RUN mkdir -p /cache
WORKDIR /app
COPY --from=builder /build/target/release/immich-edit /app/immich-edit
COPY --from=web /build/build /app/web
ENV WEB_DIR=/app/web \
    CACHE_DIR=/cache \
    BIND_ADDR=0.0.0.0:3000 \
    IMMICH_EDIT_RENDERER=auto
EXPOSE 3000
VOLUME /cache
ENTRYPOINT ["/app/immich-edit"]
