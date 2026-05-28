# Contributing

## Build locally

System dependencies (macOS):

```bash
brew install nasm jpeg-turbo libheif jpeg-xl
```

System dependencies (Debian/Ubuntu):

```bash
sudo apt-get install -y nasm cmake pkg-config libclang-dev \
  libheif-dev libjxl-dev libturbojpeg0-dev
```

Install frontend dependencies:

```bash
cd web && npm install
```

Build the Rust workspace:

```bash
cargo build --workspace
```

## Dev workflow

Copy `.env.example` to `.env`, then set `IMMICH_URL` and `IMMICH_API_KEY`. For local work, also set `BIND_ADDR=127.0.0.1:3000` or set `AUTH_TOKEN`. Run the backend and frontend in separate terminals.

Backend (Rust):

```bash
set -a
source .env
set +a
cargo run -p immich-edit-backend
```

For reloads during backend work, source `.env` the same way, then run `cargo watch -x "run -p immich-edit-backend" -w crates`.

Frontend (Vite dev server, proxies `/api` to the backend):

```bash
cd web && npm install && npm run dev
```

On macOS, run natively for GPU rendering. Metal does not pass through Docker.

## Pipeline changes

If you change RAW operators, render pass ordering, CPU/GPU ownership, or tone/output handling,
read [docs/pipeline.md](docs/pipeline.md) first and update it with the code change.

## Tests

```bash
cargo test --workspace
cd web && npm run check
```

## Commits

- Use conventional commit titles
- Examples: `feat: add crop operator`, `fix: gpu readback alignment`, `refactor: inline cpu demosaic`

## Pull requests

- Keep changes focused; one feature or fix per PR
- Add or update tests for new behavior
- CI must pass before merge

## Reporting issues

Use GitHub Issues for bugs and feature requests. For security issues, see [SECURITY.md](SECURITY.md).

## License of contributions

By submitting a PR you agree that your contribution is licensed under [AGPL-3.0-only](LICENSE), the same license as the project.
