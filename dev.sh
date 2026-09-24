#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

if [[ -f .env ]]; then
  set -a
  # shellcheck disable=SC1091
  source .env
  set +a
fi

export BIND_ADDR="${BIND_ADDR:-127.0.0.1:8088}"
export DATA_DIR="${DATA_DIR:-./data}"
export DCP_DIR="${DCP_DIR:-crates/backend/assets/dcp}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target/dev-server}"
export IMMICH_EDIT_BACKEND="${IMMICH_EDIT_BACKEND:-http://${BIND_ADDR}}"

if [[ "$(uname -s)" == Darwin ]] && command -v brew >/dev/null; then
  brew_prefix="$(brew --prefix)"
  export PKG_CONFIG_PATH="${brew_prefix}/opt/jpeg-turbo/lib/pkgconfig:${brew_prefix}/opt/libheif/lib/pkgconfig:${brew_prefix}/lib/pkgconfig${PKG_CONFIG_PATH:+:${PKG_CONFIG_PATH}}"
fi

if ! cargo watch --version >/dev/null 2>&1; then
  echo "dev.sh needs cargo-watch:" >&2
  echo "  cargo install cargo-watch --locked" >&2
  exit 1
fi

if [[ web/package-lock.json -nt web/node_modules/.package-lock.json ]]; then
  npm --prefix web ci
fi

bash web/scripts/build-wasm.sh

trap 'trap - INT TERM EXIT; kill 0 2>/dev/null || true' INT TERM EXIT

cargo watch -w crates -w Cargo.toml -w Cargo.lock -x "run -p immich-edit-backend" &
cargo watch --postpone -w crates/raw-pipeline -w crates/web-render \
  -s "bash web/scripts/build-wasm.sh" &
npm --prefix web run dev &

wait
