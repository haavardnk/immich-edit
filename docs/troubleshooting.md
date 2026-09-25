---
layout: default
title: Troubleshooting
nav_order: 5
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

The error names the replacement. See [removed settings](configuration.md#removed-settings).

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

`Access for this account is disabled.` means an administrator turned the account off under
**Settings** > **App settings** > **Users**.

An OAuth-only Immich account has no password. Use the OAuth button, or create an Immich API key
and choose **Use an Immich API key instead**.

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

The provider returns to the immich-edit page that started the sign-in, so it must know that
address. Register `/login` and `/setup` under the exact address browsers use, with the client
application Immich uses. A different scheme, port, or trailing path is enough to fail. Immich
itself has no list to update. See
[which addresses to register](oauth.md#which-addresses-to-register).

### `Single sign-on was declined`

The identity provider refused the sign-in and the message ends with its reason. Check the user's
access to the client application at the provider, then try Immich's own OAuth login with the same
account. If that fails too, the problem is in the provider or Immich's OAuth settings.

### `The sign-in link expired. Start again.`

The sign-in has to finish within ten minutes in the browser that started it. Starting a second
sign-in, in any tab, replaces the first. It also fails when a reverse proxy strips cookies from
`/api`. Start again from the login page.

### The OAuth button is missing

immich-edit asks the Immich server which sign-in methods it offers and remembers the answer for a
minute. Reload a minute after enabling OAuth in Immich. When the lookup fails, the login page shows
the password form alone rather than guessing; check that the container can reach the Immich URL.

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

## The server does not use the GPU

Open **Settings** > **Diagnostics** and read **GPU adapter** and **GPU type**. **Renderer active**
reads `gpu` even on the software rasterizer, so it does not settle the question on its own.

- **GPU type** reads `software rasterizer`: the container has no GPU. Follow
  [GPU passthrough](gpu-passthrough.md), then
  [check that it worked](gpu-passthrough.md#check-that-it-worked).
- AMD or Intel: pass `/dev/dri` and add the numeric ID of the group that owns the render node.
  A group name such as `render` does not work inside the container.
- NVIDIA: set `NVIDIA_DRIVER_CAPABILITIES` to include `graphics`. The toolkit's default gives the
  container no Vulkan driver.
- macOS Docker: Metal passthrough is unavailable. Use native execution for GPU rendering.
- Native macOS: inspect the startup log for the Metal adapter and device errors.

`IMMICH_EDIT_RENDERER=gpu` logs GPU initialization failures, then falls back to CPU.

A software rasterizer is not a broken setup. It is the expected result on a host without graphics
hardware, and it is faster than the built-in CPU renderer at preview sizes; see
[rendering](rendering.md#the-server).

## Previews do not render in the browser

**Browser renderer** in Diagnostics shows whether this browser draws previews, its GPU adapter, how
long the renderer took to load, and the last render time. When **Previews** reads `server`,
**Reason** says why:

- `browsers only offer WebGPU over HTTPS or on localhost`: the page is served over plain HTTP from
  another machine. See
  [browser previews on a local network](rendering.md#browser-previews-on-a-local-network).
- `this browser has no WebGPU`: the browser lacks WebGPU, or has it turned off.
- `Server rendering is selected`: **Settings** > **Preview rendering** is set to **Server** in this
  browser.
- Any other message: the renderer could not start, or it stopped, with the error it reported. The
  editor uses server previews for the rest of the session.

Choosing **Browser when available** again retries after a failure. Set it to **Server** to rule the
browser out when a preview looks wrong.

## Previews are slow or time out

A preview downloads the original from Immich, decodes it, then renders it. The first request for an
asset pays the download and decode; later requests reuse the cached frame, and requests that arrive
while a decode is running wait for that same decode instead of starting another one. A decode that
outlives its request still finishes and still lands in the cache, so a retry after a
`408 Request Timeout` is served from memory.

If the first preview times out, press **Retry** in the viewer. If every preview times out:

- Check **GPU adapter** and **GPU type** in Diagnostics. A software rasterizer or the CPU renderer
  is far slower than a GPU; see [the server does not use the GPU](#the-server-does-not-use-the-gpu).
- Use browser previews where you can. Slider moves then render in the browser and never wait on
  the server.
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

That is expected: ratings, favorites, tags, and reject marks belong to the Immich photo. See
[virtual copies](cull.md#virtual-copies).

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
RAW_FRAME_CACHE_MB=256
QUALITY_FRAME_CACHE_MB=256
GPU_TEXTURE_CACHE_MB=256
MASK_CACHE_MB=256
```

With one render slot, an edited thumbnail or a batch export can make the editor wait for the
current render to finish. Keep `RENDER_MAX_CONCURRENCY` at `2` or more if that wait matters more
than memory.

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
