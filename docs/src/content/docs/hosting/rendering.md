---
title: Rendering
description: How server GPU/CPU rendering and browser WebGPU previews split work and manage memory.
---

## What runs where

| Work | Runs on |
| --- | --- |
| Download and decode original from Immich | Server |
| Source settings (NR, capture sharpening, lens, retouch, DCP, crop/rotate) | Server |
| All other Develop and mask previews, including 1:1 tiles | Browser (WebGPU), else server |
| Histogram and scopes | Wherever the preview renders |
| Split-view original, exports, batch jobs, thumbnails, AI models | Server |

With browser previews active, the server sends one decoded linear source per photo; dragging tone,
color, mask, and effect sliders runs locally with zero server round-trips.

## Server rendering

The server uses Vulkan (Linux) or Metal (macOS) with an automatic CPU fallback
(`IMMICH_EDIT_RENDERER=auto|gpu|cpu`). In Docker without a hardware GPU, Mesa's `llvmpipe`
software rasterizer runs as a Vulkan device and is marked as `software rasterizer` in Diagnostics.

### GPU memory

Server renders share one GPU budget (90% of reported VRAM; integrated/unified-memory GPUs are also
capped by host RAM). Override with `GPU_MEMORY_BUDGET_MB`. Frames exceeding GPU limits render in
tiles seamlessly, or fall back to CPU for that render when no tile size fits.

### CPU memory

Host RAM (or container memory limit) minus `RAW_FRAME_CACHE_MB` and `QUALITY_FRAME_CACHE_MB` forms
the shared host budget. The CPU renderer cannot tile; frames exceeding the full budget return
`This photo is too large for the server's memory`.

## Browser previews

Browsers expose WebGPU only on secure origins (`https://` or `localhost`). Choose the per-browser
mode under **Settings** > **Preferences** > **Preview renderer**:

- **Fastest** (default): benchmarks both and picks the browser when it is at least 20% faster per
  megapixel than the server.
- **Browser**: always renders previews locally when WebGPU is available.
- **Server**: disables the browser renderer completely.

Exports always run at full resolution on the server.

### Browser previews on a local network

Plain `http://<lan-ip>:3000` is not a secure context, so browsers disable WebGPU. Pick one option:

- **Chrome / Edge flag** (client-only): open
  `chrome://flags/#unsafely-treat-insecure-origin-as-secure` (or `edge://flags/...`), add
  `http://<server-ip>:3000`, set **Enabled**, and relaunch.
- **SSH tunnel**: `ssh -N -L 3000:localhost:3000 <user>@<server>` and open `http://localhost:3000`.
- **LAN HTTPS**: use `tailscale serve --bg 3000`, or Caddy with `tls internal` behind a
  [reverse proxy](deployment.md#reverse-proxy).

If you use OAuth, register the new origin at your identity provider (see [OAuth](oauth.md)).
