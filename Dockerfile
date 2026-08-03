FROM node:26-slim AS frontend
ARG APP_VERSION=0.0.0
WORKDIR /build/web
COPY web/package.json web/package-lock.json ./
RUN npm ci --no-audit --no-fund
COPY web/ .
RUN npm version --no-git-tag-version --allow-same-version "$APP_VERSION" && \
    npm run build

FROM rust:1.96-trixie AS chef
RUN cargo install cargo-chef cargo-edit --locked
WORKDIR /build
COPY .cargo/ .cargo/

FROM chef AS planner
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS backend
ARG APP_VERSION=0.0.0
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    cmake \
    libclang-dev \
    libheif-dev \
    libheif-plugins-all \
    libjxl-dev \
    libturbojpeg0-dev \
    nasm \
    pkg-config && \
    rm -rf /var/lib/apt/lists/*
COPY --from=planner /build/recipe.json recipe.json
COPY rust-toolchain.toml ./
RUN cargo chef cook --release --tests --recipe-path recipe.json -j "$(nproc)"
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN cargo set-version --workspace "$APP_VERSION" && \
    cargo build --locked --release --bin immich-edit -j "$(nproc)" && \
    strip target/release/immich-edit && \
    mkdir -p /build/dylibs && \
    find target/release -maxdepth 1 -name '*.so*' -exec cp -L {} /build/dylibs/ \;

FROM backend AS test
RUN cargo test --locked --release --workspace -j "$(nproc)" && \
    touch /build/.tests-passed

FROM debian:trixie-slim
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libheif1 \
    libheif-plugins-all \
    libjxl0.11 \
    libturbojpeg0 \
    libvulkan1 \
    mesa-vulkan-drivers && \
    rm -rf /var/lib/apt/lists/*
RUN mkdir -p /data && \
    chown 10001:10001 /data
WORKDIR /app
COPY --from=test /build/.tests-passed /app/.tests-passed
COPY --from=backend --chown=10001:10001 /build/target/release/immich-edit /app/immich-edit
COPY --from=backend --chown=10001:10001 /build/dylibs/ /app/
COPY --from=frontend --chown=10001:10001 /build/web/build /app/web
COPY --chown=10001:10001 crates/backend/assets/dcp /app/assets/dcp
ENV WEB_DIR=/app/web \
    DCP_DIR=/app/assets/dcp \
    DATA_DIR=/data \
    BIND_ADDR=0.0.0.0:3000 \
    IMMICH_EDIT_RENDERER=auto \
    SEGMENT_RUNTIME=auto \
    XDG_CACHE_HOME=/data/cache
USER 10001:10001
EXPOSE 3000
VOLUME /data
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -fsS http://127.0.0.1:3000/api/health/live || exit 1
ENTRYPOINT ["/app/immich-edit"]
