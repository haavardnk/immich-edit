---
title: Troubleshooting
description: Quick fixes for startup, sign-in, GPU, browser preview, format, and memory issues.
---

Start with **Settings** > **Diagnostics** (**Copy support bundle** for bug reports) and server logs
(`RUST_LOG=info,http=debug` for full request logs).

## Startup and sign-in

| Symptom | Cause and fix |
| --- | --- |
| `data dir not writable` / SQLite open error | Container runs as UID/GID `10001`. Fix host bind-mount ownership or use a named Docker volume. |
| Removed setting error on start | Replace legacy v0.3 env vars; see [Removed settings](configuration.md#removed-settings). |
| `invalid value for ALLOWED_ORIGINS` | Use scheme + host (+ optional port) only, no trailing slash or path. |
| `Immich redirected the request` / `status 302` | Immich is behind an auth proxy (e.g. Cloudflare Access) or HTTP→HTTPS redirect. Use the internal URL or configure [Cloudflare Access](configuration.md#cloudflare-access). |
| `rate_limited` on login | Reverse proxy IP is missing from `TRUSTED_PROXIES`, collapsing all clients to one IP. |
| OAuth redirect URI mismatch | Register exact `<origin>/login` and `<origin>/setup` at your identity provider; see [OAuth](oauth.md). |
| OAuth button missing | Enable OAuth in Immich first; immich-edit polls within a minute. |
| `The sign-in link expired` | Complete OAuth sign-in within a few minutes and ensure `TRUSTED_PROXIES` sets `Secure` cookies properly. |
| App does not install (PWA) | Requires trusted HTTPS (not plain HTTP or clicked-through self-signed certs); see [HTTPS for the installed app](deployment.md#https-for-the-installed-app). |

## GPU and browser previews

| Symptom | Cause and fix |
| --- | --- |
| **GPU type** shows `software rasterizer` | No hardware GPU passed into Docker. Add `/dev/dri` + numeric `group_add` (AMD/Intel) or `NVIDIA_DRIVER_CAPABILITIES=graphics,utility` (NVIDIA); see [GPU passthrough](gpu.md). |
| `WebGPU needs HTTPS or localhost` | Plain HTTP LAN origin blocks browser WebGPU. Use HTTPS, Tailscale Serve, SSH tunnel, or the Chrome/Edge insecure-origin flag; see [Rendering](rendering.md#browser-previews-on-a-local-network). |
| Browser previews disabled on phone/tablet | Mobile browser lacks WebGPU or exceeded GPU texture limits; server previews take over automatically. |
| GPU device lost | Driver reset; immich-edit falls back to CPU for active renders and rebuilds the GPU device automatically. |

## Files, exports, and memory

| Symptom | Cause and fix |
| --- | --- |
| `unsupported_format` | Camera CFA or file type is unsupported; check [Supported formats](../reference/formats.md). |
| HEIC / AVIF missing | Native builds need `libheif` plugins; official Docker image includes all four codecs. |
| Export `408 Request Timeout` | Large RAW or slow link; raise `ORIGINAL_TIMEOUT_SECS` / `EXPORT_TIMEOUT_SECS` and proxy `proxy_read_timeout`. |
| `This photo is too large for the server's memory` | Exceeds host RAM budget on CPU fallback; raise container memory limit or lower `RAW_FRAME_CACHE_MB`. |
| AI mask missing or model download fails | Install models under **Administration** > **Models**; check `ML_RUNTIME` is not `off`. |
| `409 Conflict` on save | Another tab edited the same photo; reload to pick up the latest state. |
| Mask pixels disappeared | `DATA_DIR/rasters/` was cleared or excluded from restore. Restore `rasters/` with the database. |
