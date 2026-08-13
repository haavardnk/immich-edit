---
layout: default
title: Development
nav_order: 14
permalink: /development/
---

# Development

## Requirements

Install Rust from `rust-toolchain.toml`, Node 26, and the native image libraries.

macOS:

```shell
brew install nasm ninja jpeg-turbo libheif
```

Debian or Ubuntu:

```shell
sudo apt-get install -y nasm cmake ninja-build pkg-config libclang-dev \
  libturbojpeg0-dev libheif-dev libjxl-dev
```

The default backend build includes local AI-mask inference and downloads ONNX Runtime artifacts on
the first build. Use `--no-default-features` to compile the backend without ML routes.

## Run the development servers

Copy `.env.example` to `.env` and set the native development values:

```shell
BIND_ADDR=127.0.0.1:8088
DCP_DIR=crates/backend/assets/dcp
DATA_DIR=./data
```

Run the watched backend and Vite frontend:

```shell
bash dev.sh
```

Vite proxies `/api` to `127.0.0.1:8088` by default. Set `IMMICH_EDIT_BACKEND` before starting Vite
when the backend uses another address.

On macOS, run natively to use Metal. Docker cannot pass Metal through its Linux virtual machine.

## Build

```shell
cd web
npm ci
npm run build
cd ..
cargo build --workspace
```

To serve the built frontend through the Rust process:

```shell
WEB_DIR=web/build cargo run -p immich-edit-backend
```

## Preview the documentation

Install the local Pages dependencies once:

```shell
gem install --user-install jekyll jekyll-remote-theme jekyll-relative-links
gem install --user-install just-the-docs -v 0.12.0
```

Run the site from the repository root:

```shell
PATH="$(ruby -r rubygems -e 'print Gem.user_dir')/bin:$PATH" \
  jekyll serve --source docs --livereload
```

Open `http://127.0.0.1:4000/immich-edit/`.

## Test changes

Run formatting and clippy for Rust changes:

```shell
cargo fmt
cargo clippy --workspace --lib --bins --tests -- -D warnings
```

Then use the row matching the change:

| Change | Test command |
| --- | --- |
| Backend only | `cargo test -p immich-edit-backend` |
| ML only | `cargo test -p ml` |
| Raw-pipeline plumbing | `cargo test -p raw-pipeline --lib` |
| Render math, shaders, color, masks, geometry, or encode | `cargo test -p raw-pipeline --lib --tests` |
| Cargo dependencies, profiles, CI, or Dockerfile | `cargo test --workspace --lib --tests` |

Frontend changes:

```shell
cd web
npm run format
npm run lint
npm run check
npm run test:unit
```

Run focused Playwright specifications for changed user flows. A broad frontend change also runs:

```shell
npm run test:e2e
```

GPU parity tests can skip when no adapter exists. Include local GPU evidence when changing shaders,
GPU pass order, device setup, or CPU/GPU parity behavior.

Raw-pipeline integration tests share `crates/raw-pipeline/tests/common/mod.rs` for fixture discovery,
synthetic frames, JPEG decoding, and parity metrics. Declare `mod common;` and add a helper there
instead of copying one into a test binary.

Local scratch files under `crates/ml/examples` and `crates/raw-pipeline/examples` are ignored and can
break `--all-targets`. Do not add dependencies only to compile a scratch file.

## Change the product

- Keep raw-pipeline independent from HTTP, Immich, SQLite, and disk orchestration.
- Keep backend pixel work limited to orchestration.
- Use Svelte 5 runes and typed TypeScript without `any`.

Read [architecture](architecture.md) before crossing service boundaries and
[render pipeline](pipeline.md) before changing pixel operations.
