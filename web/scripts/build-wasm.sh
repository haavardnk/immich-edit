#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"

locked="$(grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | sed -n 's/^version = "\(.*\)"$/\1/p')"
installed="$(wasm-bindgen --version 2>/dev/null | cut -d' ' -f2 || true)"
if [[ "$installed" != "$locked" ]]; then
  echo "wasm-bindgen-cli $locked is required (found '${installed:-none}'):" >&2
  echo "  cargo install wasm-bindgen-cli --version $locked --locked" >&2
  exit 1
fi

target_dir="${CARGO_TARGET_DIR:-target}"
cargo build --locked --release -p web-render --target wasm32-unknown-unknown
wasm-bindgen --target web --out-name web_render --out-dir web/src/lib/wasm \
  "$target_dir/wasm32-unknown-unknown/release/web_render.wasm"
