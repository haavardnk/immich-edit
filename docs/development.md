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

The frontend needs the browser renderer in `web/src/lib/wasm`, which is generated and ignored by
Git. It needs the wasm target and the `wasm-bindgen` CLI at the version in `Cargo.lock`:

```shell
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --locked --version <the wasm-bindgen version in Cargo.lock>
```

`dev.sh` builds it before starting and rebuilds it when `crates/raw-pipeline` or
`crates/web-render` change. Outside `dev.sh`, run `npm run wasm` in `web/`. The script stops and
prints the exact install command when the installed `wasm-bindgen` does not match `Cargo.lock`.

## Run the development servers

Install `cargo-watch` once:

```shell
cargo install cargo-watch --locked
```

Run the watched backend, the watched browser renderer, and the Vite frontend:

```shell
bash dev.sh
```

`dev.sh` runs `npm ci` in `web/` whenever `package-lock.json` is newer than the installed modules,
and stops all three on Ctrl-C. It sets native defaults: the backend listens on
`127.0.0.1:8088`, keeps data in `./data`, reads camera profiles from `crates/backend/assets/dcp`,
and builds into `target/dev-server`. Put overrides in `.env`, which `dev.sh` loads; see
`.env.example`. Vite proxies `/api` to `BIND_ADDR` unless `IMMICH_EDIT_BACKEND` is set.

On macOS, run natively to use Metal. Docker cannot pass Metal through its Linux virtual machine.

## Build

```shell
cd web
npm ci
npm run wasm
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
| Browser renderer | `npm run wasm`, then `npx playwright test render-parity client-render` in `web/` |
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

`e2e/render-parity.spec.ts` runs the wasm renderer in Chromium's WebGPU and compares its canvas with
`web/e2e/fixtures/render/expected.rgb`, a native GPU render of the same source. Chromium validates
WGSL with Tint, which rejects some code that naga accepts, so run it after any shader change. On
Linux it uses SwiftShader and runs headed, so start it under a display server
(`xvfb-run -a npx playwright test render-parity`): headless Chromium on Linux destroys the WebGPU
device as soon as a canvas context is configured. Set `WEBGPU_SWIFTSHADER=1` to use SwiftShader
elsewhere. When a render change is intended, rebake the fixture with a GPU:

```shell
BAKE_WEB_PARITY=1 cargo test -p raw-pipeline --test web_parity_fixture
```

`e2e/client-render.spec.ts` opens the editor with the browser renderer on the same fixture source
and asserts that a display slider redraws the canvas with no `/preview` or `/source` request, and
that a browser without WebGPU gets server previews. Other specs pin the server renderer through
`installMocks`; pass `renderer: 'auto'` to opt in.

Raw-pipeline integration tests share `crates/raw-pipeline/tests/common/mod.rs` for fixture discovery,
synthetic frames, JPEG decoding, and parity metrics. Declare `mod common;` and add a helper there
instead of copying one into a test binary.

Local scratch files under `crates/ml/examples` and `crates/raw-pipeline/examples` are ignored and can
break `--all-targets`. Do not add dependencies only to compile a scratch file.

`docs/shortcuts.md` is generated from the `KEYBINDS` registry in `web/src/lib/keybinds.ts`. After
changing a shortcut run `npm run docs:shortcuts` in `web/` instead of editing the page;
`npm run test:unit` fails while the page and the registry disagree.

## Change the product

- Keep raw-pipeline independent from HTTP, Immich, SQLite, and disk orchestration.
- Keep backend pixel work limited to orchestration.
- Use Svelte 5 runes and typed TypeScript without `any`.

Read [architecture](architecture.md) before crossing service boundaries and
[render pipeline](pipeline.md) before changing pixel operations.
