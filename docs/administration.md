---
layout: default
title: Administration
nav_order: 10
permalink: /administration/
---

# Administration

Administration is available under **Settings**.

## Users and sessions

Users sign in with their own Immich password or API key. immich-edit provisions the local account
from the authenticated Immich user and stores no local password.

Users can revoke individual sessions or every other session. Local edits, history, presets, jobs,
and exports are private to each user. Ratings, favorites, tags, and reject marks are shared through
Immich.

An administrator can:

- Enable or disable access for another user
- Purge another user's local data
- Inspect the configured Immich instance
- Rebind immich-edit to another Immich server

Disabling access revokes that user's sessions and cancels active jobs, but retains local data.
Purging removes the user's local edits, history, presets, exports, and jobs.

{: .warning }
Rebinding the instance validates an administrator on the new server, then removes local users,
edits, and jobs and rotates the server epoch. Back up `DATA_DIR` first. The confirmation requires
the new hostname.

## Mask models

AI masks are optional. Install models under **Settings** > **Mask models**.

Installs run one at a time and verify a pinned SHA-256 digest. A failed item shows the reason and a
**Retry** action. Models are shared by all users and stored under `DATA_DIR/models`.

The catalog covers subject, people, sky, depth, scene, and click selection. Check each model's
license before commercial use. Select an installed default when a kind has more than one model.

`ML_RUNTIME=auto` tries WebGPU and falls back to CPU. `gpu` fails rather than falling back, `cpu`
skips WebGPU, and `off` disables AI masks. On arm64 Linux, inference is CPU-only because the ONNX
Runtime distribution has no WebGPU provider for that target.

## Diagnostics

Open **Settings** > **Diagnostics** to inspect:

- Server version, renderer mode, active renderer, and GPU adapter
- Immich connection state and database migration state
- HEIC and AVIF decode and encode capabilities
- Redacted configuration and resource paths
- CPU and GPU render latency
- Frame, raster, embedding, and GPU cache use

Use **Copy support bundle** when opening a Discussion or issue.

## Security

- See [Deployment](deploy.md#reverse-proxy) before exposing the service outside a trusted network.
- Session cookies are HttpOnly and SameSite Strict.
- Per-user Immich credentials are encrypted with `DATA_DIR/instance.key`.
- Metadata writes require consent in each browser before the first rating, favorite, tag, or reject
  change.

Report vulnerabilities through
[GitHub Security Advisories](https://github.com/haavardnk/immich-edit/security/advisories/new), not a
public issue.
