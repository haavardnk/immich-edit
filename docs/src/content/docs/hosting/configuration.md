---
title: Configuration
description: Environment variables, TOML configuration, trusted proxies, logging, and removed keys.
---

Immich URL and user credentials are configured in the web UI. Server settings use environment
variables or an optional TOML file (`IMMICH_EDIT_CONFIG=/path/to/config.toml`, lowercase keys).
Environment variables override TOML values.

## Settings

| Environment variable | Default | Purpose / valid values |
| --- | --- | --- |
| `BIND_ADDR` | `0.0.0.0:3000` | Listen socket |
| `DATA_DIR` | `./data` (`/data` in Docker) | Database, `instance.key`, imports, models, mask rasters |
| `DATABASE_URL` | `sqlite://DATA_DIR/immich-edit.db?mode=rwc` | SQLite connection URL |
| `PREVIEW_MAX_EDGE` | `65535` | `256`–`65535`; hard preview edge ceiling |
| `RENDER_MAX_CONCURRENCY` | `clamp(cores/2, 2, 4)` | Total concurrent renders (`> 0`) |
| `THUMB_MAX_CONCURRENCY` | `clamp(cores/4, 2, 4)` | Background thumbnails/batch renders (capped below `RENDER_MAX_CONCURRENCY`) |
| `RAW_FRAME_CACHE_MB` | `max(RENDER_MAX_CONCURRENCY * 256, 512)` | `64`–`16384`; RAM cache for preview frames |
| `QUALITY_FRAME_CACHE_MB` | `512` | `64`–`16384`; RAM cache for quality frames |
| `GPU_TEXTURE_CACHE_MB` | `clamp(gpu_budget/8, 256, 4096)` (else `512`) | `64`–`16384`; reusable GPU textures and cached stages |
| `GPU_MEMORY_BUDGET_MB` | Auto (90% VRAM) | `>= 64`; override shared GPU render budget |
| `MASK_CACHE_MB` | `512` | Disk budget for unreferenced mask rasters |
| `EMBEDDING_CACHE_MB` | `2048` | Disk budget for click-model embeddings |
| `DENOISE_CACHE_MB` | `4096` | Disk budget for AI Denoise results (newest 2 kept in RAM) |
| `IMMICH_EDIT_RENDERER` | `auto` | `auto`, `gpu`, or `cpu` (TOML key: `renderer`) |
| `GPU_TIMESTAMPS` | `false` | Record per-stage GPU device timing in Diagnostics |
| `ML_RUNTIME` | `auto` | `auto`, `gpu`, `cpu`, or `off` |
| `ML_MAX_EDGE` | `2048` | `256`–`8192`; mask inference input edge |
| `ML_MAX_CONCURRENCY` | `1` | Concurrent AI mask / denoise jobs |
| `ML_IDLE_SECS` | `60` | Idle seconds before unloading ML sessions |
| `ALLOWED_ORIGINS` | Empty | Comma-separated origins (no path or trailing slash) |
| `TRUSTED_PROXIES` | Loopback, RFC1918, link-local | Comma-separated IPs/CIDRs trusted for `X-Forwarded-*` |
| `MAX_BODY_MB` | `128` | Maximum HTTP request body size |
| `REQUEST_TIMEOUT_SECS` | `60` | Standard API request timeout |
| `ORIGINAL_TIMEOUT_SECS` | `120` | Immich original download timeout |
| `EXPORT_TIMEOUT_SECS` | `300` | Export / Immich upload timeout, and how long an export waits for an AI denoise bake |
| `CF_ACCESS_CLIENT_ID` | Empty | Cloudflare Access service token ID for Immich calls |
| `CF_ACCESS_CLIENT_SECRET` | Empty | Cloudflare Access service token secret |

## Environment-only variables

| Variable | Native default | Docker value | Purpose |
| --- | --- | --- | --- |
| `WEB_DIR` | `./web` | `/app/web` | Built frontend directory |
| `DCP_DIR` | `./assets/dcp` | `/app/assets/dcp` | Bundled DCP camera profiles |
| `IMMICH_EDIT_CONFIG` | None | None | Path to TOML config file |
| `RUST_LOG` | `info` | `info` | Log filter (targets: `http`, `upstream`, `client`, `panic`, `onnxruntime`, `immich_edit_backend`) |
| `LOG_FORMAT` | `pretty` | `pretty` | `pretty` or `json` |

## Trusted proxies

`TRUSTED_PROXIES` governs `X-Forwarded-For` (rate limiting) and `X-Forwarded-Proto` (`Secure`
session cookies). Defaults cover `127.0.0.0/8`, `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`,
`169.254.0.0/16`, `::1/128`, `fc00::/7`, and `fe80::/10`. Setting `TRUSTED_PROXIES` replaces the
defaults, so include any private ranges still needed. Set `trusted_proxies = []` in TOML to trust
no proxy.

## Cloudflare Access

If Immich sits behind Cloudflare Access, create a Service Token and add a **Service Auth** policy
on the Immich app, then set `CF_ACCESS_CLIENT_ID` and `CF_ACCESS_CLIENT_SECRET` on immich-edit.

## Removed settings

The server refuses to start if removed v0.3 keys are present:

| Removed | Replacement |
| --- | --- |
| `CACHE_DIR` | `DATA_DIR` |
| `SEGMENT_RUNTIME`, `SEGMENT_MAX_EDGE`, `SEGMENT_MAX_CONCURRENCY`, `SEGMENT_IDLE_SECS` | `ML_RUNTIME`, `ML_MAX_EDGE`, `ML_MAX_CONCURRENCY`, `ML_IDLE_SECS` |
| `IMMICH_URL`, `IMMICH_API_KEY`, `AUTH_TOKEN`, `IMMICH_EDIT_INSECURE` | Configured via first-run setup UI |
