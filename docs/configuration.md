---
layout: default
title: Configuration
nav_order: 9
permalink: /configuration/
---

# Configuration

The Immich URL and user credentials are configured in the app. Infrastructure settings use
environment variables or an optional TOML file. Environment variables override matching TOML
values.

Set `IMMICH_EDIT_CONFIG=/path/to/config.toml` to load a file. TOML keys use lowercase names, such
as `render_max_concurrency` for `RENDER_MAX_CONCURRENCY`.

## Settings

| Environment variable | Default | Valid values or purpose |
| --- | --- | --- |
| `BIND_ADDR` | `0.0.0.0:3000` | Listen socket |
| `DATA_DIR` | `./data` | Durable database, key, imports, models, and mask rasters; Docker sets `/data` |
| `DATABASE_URL` | `sqlite://DATA_DIR/immich-edit.db?mode=rwc` | SQLite connection URL |
| `PREVIEW_MAX_EDGE` | `65535` | `256` to `65535`; hard ceiling for preview requests |
| `RENDER_MAX_CONCURRENCY` | Half the CPU cores, clamped to `2` through `4` | Interactive preview and export renders; must be nonzero |
| `THUMB_MAX_CONCURRENCY` | Quarter of the CPU cores, clamped to `2` through `4` | Edited-thumbnail renders; must be nonzero |
| `RAW_FRAME_CACHE_MB` | `max(RENDER_MAX_CONCURRENCY * 256, 512)` | `64` to `16384`; decoded preview frames in RAM |
| `QUALITY_FRAME_CACHE_MB` | `512` | `64` to `16384`; decoded quality frames in RAM |
| `GPU_TEXTURE_CACHE_MB` | `512` | `64` to `16384`; reusable GPU textures |
| `MASK_CACHE_MB` | `512` | Disk budget for unreferenced mask rasters; must be nonzero |
| `EMBEDDING_CACHE_MB` | `2048` | Disk budget for click-model embeddings; must be nonzero |
| `IMMICH_EDIT_RENDERER` | `auto` | `auto`, `gpu`, or `cpu` |
| `ML_RUNTIME` | `auto` | `auto`, `gpu`, `cpu`, or `off` |
| `ML_MAX_EDGE` | `2048` | `256` to `8192`; source edge sent to mask inference |
| `ML_MAX_CONCURRENCY` | `1` | Concurrent inference jobs; must be nonzero |
| `ML_IDLE_SECS` | `60` | Seconds before an inactive model session unloads |
| `ALLOWED_ORIGINS` | Empty | Comma-separated HTTP or HTTPS origins without a path or trailing slash |
| `MAX_BODY_MB` | `128` | Maximum request body size; must be nonzero |
| `REQUEST_TIMEOUT_SECS` | `60` | Timeout for ordinary API requests; must be nonzero. Export requests use `ORIGINAL_TIMEOUT_SECS` plus `EXPORT_TIMEOUT_SECS` instead |
| `ORIGINAL_TIMEOUT_SECS` | `120` | Immich original-download timeout |
| `EXPORT_TIMEOUT_SECS` | `300` | Immich export/upload timeout |

## Environment-only paths

These settings do not have TOML equivalents:

| Variable | Native default | Docker value | Purpose |
| --- | --- | --- | --- |
| `WEB_DIR` | `./web` | `/app/web` | Built static frontend |
| `DCP_DIR` | `./assets/dcp` | `/app/assets/dcp` | Bundled camera profiles imported at startup |
| `IMMICH_EDIT_CONFIG` | None | None | TOML configuration path |
| `RUST_LOG` | Application default | Application default | Rust tracing filter |

## Example TOML

```toml
bind_addr = "127.0.0.1:3000"
data_dir = "/srv/immich-edit"
renderer = "auto"
ml_runtime = "auto"
render_max_concurrency = 2
allowed_origins = ["https://edit.example.com"]
```

## Removed settings

v0.4.0 refuses to start when a removed key is present. The error names the replacement.

| Removed | Replacement |
| --- | --- |
| `CACHE_DIR` | `DATA_DIR` |
| `SEGMENT_RUNTIME` | `ML_RUNTIME` |
| `SEGMENT_MAX_EDGE` | `ML_MAX_EDGE` |
| `SEGMENT_MAX_CONCURRENCY` | `ML_MAX_CONCURRENCY` |
| `SEGMENT_IDLE_SECS` | `ML_IDLE_SECS` |

`IMMICH_URL`, `IMMICH_API_KEY`, `AUTH_TOKEN`, and `IMMICH_EDIT_INSECURE` are also no longer read.
Configure the Immich connection through first-run setup.
