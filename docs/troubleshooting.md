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

Password login is unavailable for an OAuth-only Immich account. Create an Immich API key and use
the API-key option. An administrator can also disable local access under **Settings**.

Repeated failures trigger temporary rate limiting.

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

| Capability | Debian package |
| --- | --- |
| HEIC decode | `libheif-plugin-libde265` |
| HEIC encode | `libheif-plugin-x265` |
| AVIF decode | `libheif-plugin-dav1d` |
| AVIF encode | `libheif-plugin-aomenc` or `libheif-plugin-rav1e` |

The official image installs `libheif-plugins-all`. Native and custom-image deployments must install
the required plugins themselves.

## Rendering uses the CPU

Open **Settings** > **Diagnostics** and inspect the active renderer and GPU adapter.

- AMD or Intel Docker: pass `/dev/dri` and the group that owns the render node.
- NVIDIA Docker: install NVIDIA Container Toolkit and add the GPU reservation.
- macOS Docker: Metal passthrough is unavailable. Use native execution for GPU rendering.
- Native macOS: inspect the startup log for the Metal adapter and device errors.

`IMMICH_EDIT_RENDERER=gpu` logs GPU initialization failures, then falls back to CPU.

## The GPU device is lost

The backend falls back to CPU for active renders and later attempts to rebuild the GPU renderer.
Restart the service after a driver crash. If device loss repeats under memory pressure, lower
`RENDER_MAX_CONCURRENCY` and `GPU_TEXTURE_CACHE_MB`.

## AI masks are unavailable

### A mask type is missing

An administrator must install that model kind under **Settings** > **Mask models**. Check
`ML_RUNTIME`; `off` disables every generated mask.

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
