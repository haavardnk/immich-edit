FROM node:22-slim AS web
WORKDIR /build
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web/ .
RUN npm run build

FROM rust:1.92-trixie AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    nasm cmake pkg-config libclang-dev \
    libheif-dev libjxl-dev libturbojpeg0-dev && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /build
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/
RUN cargo build --release --bin immich-edit -j$(nproc) && \
    strip target/release/immich-edit

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libturbojpeg0 \
    libheif1 \
    libheif-plugins-all \
    libjxl0.11 \
    libvulkan1 \
    mesa-vulkan-drivers && \
    rm -rf /var/lib/apt/lists/*
RUN groupadd --system --gid 10001 immich-edit && \
    useradd --system --uid 10001 --gid 10001 --home-dir /app --shell /usr/sbin/nologin immich-edit && \
    mkdir -p /cache /app && \
    chown -R immich-edit:immich-edit /cache /app
WORKDIR /app
COPY --from=builder --chown=immich-edit:immich-edit /build/target/release/immich-edit /app/immich-edit
COPY --from=web --chown=immich-edit:immich-edit /build/build /app/web
ENV WEB_DIR=/app/web \
    CACHE_DIR=/cache \
    BIND_ADDR=0.0.0.0:3000 \
    IMMICH_EDIT_RENDERER=auto
USER immich-edit:immich-edit
EXPOSE 3000
VOLUME /cache
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -fsS http://127.0.0.1:3000/api/health/live || exit 1
LABEL org.opencontainers.image.title="immich-edit" \
      org.opencontainers.image.description="Self-hosted non-destructive RAW photo editor for Immich" \
      org.opencontainers.image.licenses="AGPL-3.0-or-later" \
      org.opencontainers.image.source="https://github.com/haavardnk/immich-edit"
ENTRYPOINT ["/app/immich-edit"]
