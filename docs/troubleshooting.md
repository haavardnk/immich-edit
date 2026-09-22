---
layout: default
title: Troubleshooting
nav_order: 12
permalink: /troubleshooting/
---

# Troubleshooting

Start with **Settings** > **Diagnostics** and the server log. Error responses include a request ID
that identifies the matching log span.

## The server does not start

### `data dir not writable`

The process cannot create or write `DATA_DIR`. The official container runs as UID and GID `10001`.
Fix the bind-mount ownership or use a named volume.

### A removed setting is reported

v0.4.0 refuses `CACHE_DIR` and `SEGMENT_*`. Replace them with `DATA_DIR` and `ML_*`. Remove old
Immich URL or shared credential variables and configure the connection in the browser.

### SQLite cannot open the database

Confirm `DATA_DIR` exists and is writable. A custom `DATABASE_URL` should include `?mode=rwc` when
SQLite must create the file.

### `invalid value for ALLOWED_ORIGINS`

Use exact HTTP or HTTPS origins without paths or trailing slashes:

```text
https://edit.example.com
```

Separate several environment values with commas.

## Setup or sign-in fails

### Setup stays on the first screen

Setup requires an Immich administrator. Confirm the Immich URL is reachable from the immich-edit
container and the selected account is an administrator.

### `immich url host is not allowed`

The URL points at a link-local, cloud metadata, unspecified, multicast, or credential-bearing
address. Use the normal hostname, loopback address, or private network address.

### A valid user cannot sign in

An OAuth-only Immich account has no password. Use the OAuth button, or create an Immich API key and
use the API-key option. An administrator can also disable local access under **Settings**.

### `rate_limited`

Sign-in, setup, API-key and OAuth-callback requests answer `429 Too Many Requests` with the error
code `rate_limited` and a `Retry-After` header once failures pile up. Five failures from the same
address for the same account lock that pair for 15 minutes, and 25 failures against one account
from any address lock the account for an hour. Only failures count; a successful sign-in clears
both counters immediately.

Wait out the `Retry-After` value rather than retrying. If a working password is being rejected
first, fix that before the limiter hides it. The counters live in memory, so restarting the
container also clears them.

### The identity provider reports a redirect URI mismatch

immich-edit returns from the provider to the page that started the flow. Register
`https://<edit-host>/login` and `https://<edit-host>/setup` as redirect URIs in Immich and with the
provider, using the address browsers actually use. A mismatch in scheme, port, or trailing path is
enough to fail.

### `The sign-in link expired. Start again.`

The flow cookie lives for ten minutes and is bound to one browser. It also breaks when the tab that
started the flow is not the tab that returns, or when a reverse proxy strips cookies from `/api`.
Start the sign-in again from the login page.

### The OAuth button is missing

immich-edit asks the Immich server which methods it offers. When that lookup fails it shows the
password form alone rather than guessing. Check that the container can reach the Immich URL, then
reload.

## Immich is unavailable

Diagnostics distinguishes these states:

- `unreachable`: check the URL, Immich process, DNS, and Docker networks.
- `api_key_rejected`: sign out and authenticate again. Replace an expired or revoked API key.
- `timeout`: raise `ORIGINAL_TIMEOUT_SECS` for original downloads or `EXPORT_TIMEOUT_SECS` for
  uploads after checking Immich performance.
- `upstream_5xx`: inspect Immich and its reverse-proxy logs.

Smart search can fail while the rest of Immich works because it also depends on Immich machine
learning. immich-edit falls back to filename search in that case.

## A file does not open or export

### `unsupported_format`

Check [compatibility](compatibility.md). Include the camera make, model, extension, and a sample
file when reporting a decoder gap. Converting to DNG can provide a temporary workaround for some
RAW formats.

### HEIC or AVIF capabilities are missing

libheif uses separate plugins for each operation:

| Capability  | Debian package                                    |
| ----------- | ------------------------------------------------- |
| HEIC decode | `libheif-plugin-libde265`                         |
| HEIC encode | `libheif-plugin-x265`                             |
| AVIF decode | `libheif-plugin-dav1d`                            |
| AVIF encode | `libheif-plugin-aomenc` or `libheif-plugin-rav1e` |

The official image installs `libheif-plugins-all`. Native and custom-image deployments must install
the required plugins themselves.

### An export returns `408 Request Timeout`

A full-resolution export decodes, renders, encodes, and uploads inside a single request. On CPU-only
hardware a large file can take several minutes. Exports are allowed `ORIGINAL_TIMEOUT_SECS` plus
`EXPORT_TIMEOUT_SECS`; raise both if exports are cut short. A reverse proxy in front of immich-edit
needs a read timeout at least as long — see [deploy](deploy.md).

## Rendering uses the CPU

Open **Settings** > **Diagnostics** and inspect the active renderer and GPU adapter.

- AMD or Intel Docker: pass `/dev/dri` and the group that owns the render node.
- NVIDIA Docker: install NVIDIA Container Toolkit and add the GPU reservation.
- macOS Docker: Metal passthrough is unavailable. Use native execution for GPU rendering.
- Native macOS: inspect the startup log for the Metal adapter and device errors.

`IMMICH_EDIT_RENDERER=gpu` logs GPU initialization failures, then falls back to CPU.

### Diagnostics reports a software rasterizer

When no graphics hardware is present, Mesa's llvmpipe (or lavapipe) advertises itself as a Vulkan
device and runs the shaders on the CPU. The backend uses it, and Diagnostics labels it **GPU type:
software rasterizer** so the adapter name in the row above is not mistaken for real hardware.

That is expected rather than a broken setup: the Docker image ships Mesa so Intel and AMD
passthrough works, and Mesa always registers llvmpipe as a fallback. Measured on a 6-core arm64
container, llvmpipe rendered a 6000x4000 frame faster than the built-in CPU renderer at preview
sizes — 0.54x the time at 1024 px, 0.72x at 2048 px — and reached parity at 4096 px. Hardware
acceleration is still much faster than either; a software rasterizer only means the host has no GPU
to pass through.

## Previews are slow or time out

A preview downloads the original from Immich, decodes it, then renders it. The first request for an
asset pays the download and decode; later requests reuse the cached frame, and requests that arrive
while a decode is running wait for that same decode instead of starting another one. A decode that
outlives its request still finishes and still lands in the cache, so a retry after a
`408 Request Timeout` is served from memory.

If the first preview times out, press **Retry** in the viewer. If every preview times out:

- Check Diagnostics for the active renderer. A skipped or missing GPU means CPU rendering.
- Raise `REQUEST_TIMEOUT_SECS` on a slow host with large RAW files.
- Give the frame cache enough room for the files in use (`RAW_FRAME_CACHE_MB`).

## The GPU device is lost

The backend falls back to CPU for active renders and later attempts to rebuild the GPU renderer.
Restart the service after a driver crash. If device loss repeats under memory pressure, lower
`RENDER_MAX_CONCURRENCY` and `GPU_TEXTURE_CACHE_MB`.

## AI masks are unavailable

### A mask type is missing

An administrator must install that model kind under **Settings** > **App settings** > **Mask models**.
Check `ML_RUNTIME`; `off` disables every generated mask.

### Model installation fails

The server needs outbound HTTPS to the model source and space under `DATA_DIR/models`. The installer
rejects incomplete or changed files when the SHA-256 digest differs.

### Inference is slow

The first request loads a model. `ML_RUNTIME=auto` tries WebGPU and falls back to CPU. On arm64
Linux, CPU inference is expected. Lower-memory model alternatives are available in Settings.

## A save returns `409 Conflict`

Two tabs changed the same edit record. The losing tab receives the current server version. There is
no automatic merge. Keep one tab's changes or reapply them after loading the current state.

## A virtual copy shares metadata

Ratings, favorites, tags, and reject marks belong to the underlying Immich asset and appear on every
copy. Edits, masks, history, and export jobs belong to one local version.

## Mask pixels disappeared

Brush and generated masks live under `DATA_DIR/rasters`. The app protects referenced files from its
own eviction, but manual deletion still destroys them. Restore the database and rasters from the
same backup.

## Rendering is slow or memory grows

The first render decodes the original and fills caches. Later edits reuse the decoded frame.

**Render stages** in Diagnostics shows where the time goes, with typical (p50) and slow (p95)
times for each stage. **Source frames** covers the Immich download and the decode, which a cached
frame skips. On the GPU renderer the wall time of a shader stage only covers recording the work;
the device runs it later, and that wait shows up under `readback`. Set `GPU_TIMESTAMPS=true` to
add a GPU column with the device time of each stage. On the GPU renderer `histogram`, `scopes`,
and `encode` run in parallel, so their times overlap rather than add up.

Use **Settings** > **Diagnostics** before changing budgets. For a constrained CPU-only host, start
with:

```text
RENDER_MAX_CONCURRENCY=1
THUMB_MAX_CONCURRENCY=1
RAW_FRAME_CACHE_MB=256
QUALITY_FRAME_CACHE_MB=256
GPU_TEXTURE_CACHE_MB=256
MASK_CACHE_MB=256
```

Reducing `PREVIEW_MAX_EDGE` limits image detail and should be a last resort. GPU rendering remains
the recommended path for large RAW files.

## Increase log detail

Set:

```shell
RUST_LOG=immich_edit_backend=debug,tower_http=debug
```

For unresolved usage or setup questions, open a
[GitHub Discussion](https://github.com/haavardnk/immich-edit/discussions). For a reproducible bug,
open an [issue](https://github.com/haavardnk/immich-edit/issues) with the version, deployment method,
renderer and adapter from Diagnostics, request ID, relevant logs, camera and file format, and exact
reproduction steps.
