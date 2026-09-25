---
layout: default
title: Administration
parent: Run the server
nav_order: 5
permalink: /administration/
---

# Administration

Open **Settings** from the gear in the top bar. Every user sees **Account**, with
**Signed-in devices** inside it, and **Preview rendering**, plus **Diagnostics** in the sidebar.
Administrators also get **App settings** with **Users**, **Mask models** and **Immich instance**.
Sections stay collapsed until you open them.

## Users

Every immich-edit user is an Immich user. They sign in with OAuth, their Immich email and password,
or an Immich API key, and immich-edit creates the local account from the Immich user it gets back.
It stores no password of its own. Administrator rights follow the user's Immich administrator flag.
OAuth has its own page: [OAuth sign-in](oauth.md).

Local edits, history, presets, jobs, and exports are private to each user. Ratings, favorites,
tags, and reject marks are shared through Immich.

Under **App settings** > **Users**, an administrator can:

- Enable or disable access for another user. Disabling revokes that user's sessions and cancels
  active jobs, but keeps their local data.
- Purge another user's local data: edits, history, presets, exports, and jobs.

## Signed-in devices

**Account** > **Signed-in devices** lists each browser signed in to your account, with its IP
address and when it was last seen. The browser you are using has a `this session` badge.
**Revoke session** signs one device out, and **Revoke all other sessions** signs out every device
but this one. Signing out of a password or OAuth session also revokes the Immich token behind it.

## Immich instance

**App settings** > **Immich instance** shows the Immich server immich-edit is bound to, and can
rebind it to another one.

{: .warning }
Rebinding validates an administrator on the new server, then removes local users, edits, and jobs
and rotates the server epoch. Back up `DATA_DIR` first. The confirmation requires the new hostname.
Rebinding takes an email and password or an API key; an OAuth-only administrator creates an Immich
API key for it.

## Mask models

AI masks are optional. Install models under **App settings** > **Mask models**.

Installs run one at a time and verify a pinned SHA-256 digest. A failed item shows the reason and a
**Retry** action. Models are shared by all users and stored under `DATA_DIR/models`.

The catalog covers subject, people, sky, depth, scene, and click selection. Check each model's
license before commercial use. Select an installed default when a kind has more than one model.

`ML_RUNTIME=auto` tries WebGPU and falls back to CPU. `gpu` fails rather than falling back, `cpu`
skips WebGPU, and `off` disables AI masks. On arm64 Linux, inference is CPU-only because the ONNX
Runtime distribution has no WebGPU provider for that target.

## LUTs and camera profiles

Imported 3D LUTs and DCP camera profiles are shared by every user, so only an administrator can
import or remove them. Both happen in the editor, not in Settings:

- **Develop** > **LUT**: **Import .cube LUT**, and **Delete LUT** for the selected one.
- **Develop** > **Camera Profile**: **Import .dcp profile**, and **Delete imported profile** for the
  selected one. Bundled profiles cannot be deleted.

All users can browse and apply them. The files live under `DATA_DIR/luts` and `DATA_DIR/dcp`.

## Diagnostics

**Settings** > **Diagnostics** is the first place to look when something is slow or broken:

| Section | What it shows |
| --- | --- |
| **Server** | Version, **Renderer mode**, **Renderer active**, **GPU adapter**, **GPU type** on a software rasterizer, Immich connection state, and database state |
| **Browser renderer** | Whether this browser draws **Previews**, the **Reason** when it does not, its **GPU adapter**, and **Wasm load** and **Last render** times |
| **Host** | OS, CPU model, cores available to the process, and memory with any container limit |
| **HEIF codecs** | HEIC and AVIF decode and export support |
| **Render latency** | Typical and slow CPU and GPU render times |
| **Render stages**, **Source frames** | Time per pipeline stage, original download and decode times, frame cache hit rate, and requests cut off by the timeout |
| **Cache usage**, **GPU memory pools** | Frame, raster, and GPU memory use against their limits |
| **Configuration** | The effective configuration, with secrets redacted |

[Check that it worked](gpu-passthrough.md#check-that-it-worked) explains the GPU rows, and
[previews do not render in the browser](troubleshooting.md#previews-do-not-render-in-the-browser)
lists the browser renderer reasons. Use **Copy support bundle** when opening a Discussion or issue.

## Security

- See [reverse proxy](deploy.md#reverse-proxy) before exposing the service outside a trusted
  network.
- Session cookies are HttpOnly and SameSite Strict.
- Per-user Immich credentials are encrypted with `DATA_DIR/instance.key`.
- Metadata writes require consent in each browser before the first rating, favorite, tag, or reject
  change.

Report vulnerabilities through
[GitHub Security Advisories](https://github.com/haavardnk/immich-edit/security/advisories/new), not a
public issue.
