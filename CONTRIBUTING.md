# Contributing

## Build

System dependencies (macOS):

```bash
brew install nasm jpeg-turbo libheif jpeg-xl
```

System dependencies (Debian/Ubuntu):

```bash
sudo apt-get install -y nasm cmake pkg-config libclang-dev \
  libheif-dev libjxl-dev libturbojpeg0-dev
```

Frontend:

```bash
cd web && npm install
```

Backend:

```bash
cargo build --workspace
```

## Dev workflow

Backend (Rust):

```bash
cargo run -p backend
```

Frontend (SvelteKit dev server, proxies `/api` to backend):

```bash
cd web && npm run dev
```

Containerized dev setup (file watching, copy `docker-compose.example.yml` first):

```bash
cp docker-compose.example.yml docker-compose.dev.yml
docker compose -f docker-compose.dev.yml up
```

## Tests

```bash
cargo test --workspace
cd web && npm run check
```

RAW test fixtures live under `crates/raw-pipeline/tests/fixtures/`. Tests skip gracefully if a fixture is missing.

## Code style

Rust:

- No comments or docstrings
- Fully typed signatures
- Early returns over nesting; no asserts
- Run `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings` before pushing

TypeScript / Svelte:

- No comments; no `any`
- Svelte 5 runes (`$state`, `$derived`, `$effect`)
- `npx svelte-check` must be zero errors and warnings

## Commits

- Conventional commit titles only, no body
- Under 50 characters
- Examples: `feat: add crop operator`, `fix: gpu readback alignment`, `refactor: inline cpu demosaic`

## Pull requests

- Keep changes focused; one feature or fix per PR
- Add or update tests for new behavior
- CI must pass before merge

## Reporting issues

Bug reports and feature requests are welcome via GitHub Issues. For security issues, see [SECURITY.md](SECURITY.md).
