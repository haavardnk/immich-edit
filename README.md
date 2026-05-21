# immich-edit

Self-hosted RAW photo editor for [Immich](https://immich.app/). Browse your library, render RAW files to JPEG previews with GPU acceleration, apply non-destructive edits, and export full-resolution JPEGs.

Edits are stored locally in SQLite — Immich is never modified.

## Features

- Browse albums, folders, people, tags, favorites
- Exposure, contrast, highlights, shadows, saturation, white balance
- Rotate, flip, crop
- Live preview with GPU acceleration (CPU fallback always available)
- Export edited full-resolution JPEGs
- Single container, dark UI

## Quick start

```bash
docker run --rm -p 3000:3000 \
  -e IMMICH_URL=http://your-immich:2283 \
  -e IMMICH_API_KEY=your-key \
  -v immich-edit-cache:/cache \
  immich-edit:local
```

Open `http://localhost:3000`.

## Configuration

All options can be set via environment variables or a TOML config file (set `IMMICH_EDIT_CONFIG` to its path).

| Variable | Default | Description |
|---|---|---|
| `IMMICH_URL` | *(required)* | Immich server URL |
| `IMMICH_API_KEY` | *(required)* | Immich API key |
| `BIND_ADDR` | `0.0.0.0:3000` | Listen address |
| `CACHE_DIR` | `./cache` | Cache and database directory |
| `PREVIEW_MAX_EDGE` | `2048` | Max preview dimension (256–8192) |
| `RENDER_MAX_CONCURRENCY` | `2` | Parallel render limit |
| `IMMICH_EDIT_RENDERER` | `auto` | `auto`, `cpu`, or `gpu` |
| `DATABASE_URL` | `sqlite://CACHE_DIR/immich-edit.db` | SQLite connection string |

See `config.example.toml` for the file format.

## GPU acceleration

wgpu selects the best available backend automatically:

| Platform | Backend | Setup |
|---|---|---|
| Linux AMD | Vulkan | `--device /dev/dri` + `mesa-vulkan-drivers` (included in image) |
| Linux NVIDIA | Vulkan | `nvidia-container-toolkit` + deploy config in compose |
| macOS (native) | Metal | Works out of the box when running natively |

Set `IMMICH_EDIT_RENDERER=cpu` to force CPU mode. `auto` (default) tries GPU first and falls back to CPU on failure.

Check `GET /api/health` to see which renderer is active and the GPU adapter name.

## Development

```bash
# Backend (Rust)
cargo run -p backend

# Frontend (SvelteKit dev server, proxies /api to backend)
cd web && npm install && npm run dev

# Tests
cargo test --workspace
cd web && npx svelte-check
```

Or use `docker compose -f docker-compose.dev.yml up` for a containerized dev setup with file watching.

## Building

```bash
# Docker image
docker build -t immich-edit:local .

# Native
cargo build --release --bin immich-edit
cd web && npm ci && npm run build
# Point WEB_DIR at web/build when running the binary
```

## License

AGPL-3.0-only
