# Contributing

## Build from source

System dependencies (macOS):

```bash
brew install nasm jpeg-turbo libheif jpeg-xl
```

System dependencies (Debian/Ubuntu):

```bash
sudo apt-get install -y nasm cmake pkg-config libclang-dev \
  libheif-dev libjxl-dev libturbojpeg0-dev
```

Install and build the frontend:

```bash
cd web
npm ci
npm run build
cd ..
```

Build the Rust workspace, including the RAW pipeline crate:

```bash
cargo build --workspace
```

The default backend build includes local AI mask inference and downloads the pinned ONNX Runtime and WebGPU runtime artifacts during the first build. To compile the backend without that feature, use `cargo build -p immich-edit-backend --no-default-features`.

## Dev workflow

Set `BIND_ADDR=127.0.0.1:8088` and `DCP_DIR=crates/backend/assets/dcp` (via env or a `.env` copied from `.env.example`). Vite proxies `/api` there by default. Run the backend and frontend in separate terminals, then complete the in-app setup wizard with your Immich server URL and an Immich admin account.

Backend (Rust):

```bash
set -a
source .env
set +a
cargo run -p immich-edit-backend
```

To serve the built frontend through the Rust server instead of Vite, first run `npm run build` in `web/`. Then, from the repository root after sourcing `.env`, run `WEB_DIR=web/build cargo run -p immich-edit-backend`.

For reloads during backend work, install `cargo-watch`, source `.env` the same way, then run `cargo watch -x "run -p immich-edit-backend" -w crates`.

Frontend (Vite dev server):

```bash
cd web
npm ci
npm run dev
```

If you bind the backend somewhere else, set `IMMICH_EDIT_BACKEND` before starting Vite.

On macOS, run natively for GPU rendering. Metal does not pass through Docker.

## Pipeline changes

If you change RAW operators, render pass ordering, CPU/GPU ownership, or tone/output handling,
read [docs/pipeline.md](docs/pipeline.md) first and update it with the code change.

## Tests

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd web && npm run check && npm run build && npm run test:unit && npm run test:e2e
```

Most of the Rust test time sits in three `raw-pipeline` binaries — `parity_metrics`, `cpu_render` and `cpu_cache`. While iterating, scope to the package you are changing (`cargo test -p immich-edit-backend`, `cargo test -p ml`, `cargo test -p raw-pipeline --lib`) and run the full workspace before you push.

Frontend changes need coverage: unit tests (Vitest, `web/src/**/*.test.ts`) for pure logic and Playwright e2e (`web/e2e/`) for user flows.

GPU parity tests require local GPU hardware and do not run on GitHub-hosted Ubuntu. If a PR changes GPU shaders, render pass ordering, GPU/CPU parity behavior, or device setup, include local GPU test evidence in the PR description.

## Commits

- Use conventional commit titles
- Examples: `feat: add crop operator`, `fix: gpu readback alignment`, `refactor: inline cpu demosaic`

## Pull requests

- Keep changes focused; one feature or fix per PR
- Add or update tests for new behavior (Rust tests, Vitest, or Playwright)
- CI must pass before merge

## Reporting issues

Use GitHub Issues for bugs and feature requests. For security issues, see [SECURITY.md](SECURITY.md).

## License of contributions

By submitting a PR you agree that your contribution is licensed under [AGPL-3.0-only](LICENSE), the same license as the project.
