---
layout: default
title: Rendering
parent: Run the server
nav_order: 3
permalink: /rendering/
---

# Rendering

An edit can be drawn in two places: on the immich-edit server, and in the browser you edit from.
Each has its own GPU, and they do different jobs. Knowing which one does what tells you where a
graphics card helps.

## What runs where

| Work | Runs on |
| --- | --- |
| Downloading and decoding the original from Immich | Server |
| Source settings: noise reduction, capture sharpening, lens correction, retouch, camera profile, crop and rotation | Server |
| Every other Develop and mask adjustment in the editor preview, including 1:1 tiles | Browser with WebGPU, otherwise server |
| Histogram and scopes | Wherever the preview was drawn |
| The original in split view | Server |
| Exports, batch jobs and edited thumbnails | Server |
| AI mask models | Server |

With browser previews, the server prepares the photo once: it decodes the original, applies the
source settings, and sends the result to the browser. The browser applies everything else locally,
so dragging exposure, white balance, tone, color, mask or effect sliders sends nothing to the
server. Changing a source setting asks the server for a new copy, and the preview keeps using the
old one until it arrives.

## The server

The server renders on a GPU through Vulkan on Linux or Metal on macOS, and falls back to a built-in
CPU renderer. `IMMICH_EDIT_RENDERER` chooses between them; see [configuration](configuration.md).

Inside Docker, the server can only use a GPU that is passed through to the container. See
[GPU passthrough](gpu-passthrough.md).

Without a GPU, the server does not drop straight to its CPU renderer. The image ships Mesa, whose
llvmpipe driver presents itself as a Vulkan device and runs the GPU code on CPU cores.
**Settings** > **Diagnostics** then shows **GPU type** as `software rasterizer`. This is expected
on a host with no graphics card. On a 6-core arm64 container, llvmpipe rendered a 6000x4000 frame
in 0.54x the built-in CPU renderer's time at 1024 px and 0.72x at 2048 px, and matched it at
4096 px. Real graphics hardware is much faster than either.

A server GPU still matters with browser previews on. It speeds up:

- Opening a photo and changing a source setting
- Exports, batch jobs and edited thumbnails
- Previews for browsers that render on the server
- AI masks

On a CPU-only server, browser previews keep slider feedback fast. The waits move to opening a
photo, changing a source setting, and exporting.

## The browser

The editor draws previews itself when the browser supports WebGPU and treats the page as secure.
Browsers only offer WebGPU to secure pages: HTTPS, or `localhost` on the same machine. Opening
immich-edit at `http://<server-address>:3000` from another computer is not secure in that sense,
so it uses server previews. Everything still works; slider moves just wait on the server. The next
section lists ways around it.

The browser renderer uses the GPU of the computer you edit from, not the server's. It loads when
the editor opens. Each browser keeps its own choice under
**Settings** > **Preview rendering**:

- **Browser when available** draws previews locally and falls back to the server when it cannot.
- **Server** always asks the server.

When the browser renderer fails, the editor switches to server previews for the rest of the
session. Choosing **Browser when available** again retries. **Settings** > **Diagnostics** shows
the browser renderer's state, its GPU adapter and the reason for any fallback; see
[troubleshooting](troubleshooting.md#previews-do-not-render-in-the-browser).

Exports never use the browser. The server renders them from the original at full resolution.

### Browser previews on a local network

This is a browser rule, not an immich-edit setting, so the fix is on the browser or network side.
Pick whichever suits your setup. Each one also changes the address you open, so if you sign in
with OAuth, [register that address](oauth.md#which-addresses-to-register) at the identity provider.

**Chrome or Edge: allow your server's address.** Takes a minute and needs nothing on the server.

1. Open `chrome://flags/#unsafely-treat-insecure-origin-as-secure`, or
   `edge://flags/#unsafely-treat-insecure-origin-as-secure` in Edge.
1. Enter the exact address you use, such as `http://192.168.1.20:3000`, and set the flag to
   **Enabled**.
1. Relaunch the browser.

Repeat it on each computer and browser you edit from. The flag only covers the addresses you
list, so only add your own server. Safari has no equivalent.

**An SSH tunnel: use `localhost`.** Works in every browser, for one computer at a time:

```shell
ssh -N -L 3000:localhost:3000 <user>@<server-address>
```

Then open `http://localhost:3000` on that computer.

**HTTPS on your network.** Works in every browser and on every device:

- [Tailscale Serve](https://tailscale.com/kb/1312/serve) gives the server an HTTPS address with a
  trusted certificate inside your tailnet: run `tailscale serve --bg 3000` on the host.
- A [reverse proxy](deploy.md#reverse-proxy) such as Caddy issues its own certificate for a LAN
  address. Browsers warn about it until you install Caddy's root certificate on your computers;
  Chrome and Edge run browser previews once you accept the warning.

  ```caddyfile
  https://192.168.1.20 {
    tls internal
    reverse_proxy 127.0.0.1:3000
  }
  ```

**Settings** > **Diagnostics** shows `browsers only offer WebGPU over HTTPS or on localhost` as the
browser renderer's reason while none of these is in place.
