---
title: Deployment
description: Docker tags, reverse proxy TLS, HTTPS for PWA install, health checks, and native builds.
---

Use the Docker image (`haavardnk/immich-edit`, Linux `amd64`/`arm64`, UID/GID `10001`) for Linux
servers. On macOS, run natively to use Metal.

## Image tags

- `X.Y.Z` — pinned release (recommended for production)
- `X.Y`, `X`, `latest` — newest stable release
- `edge` — latest `main` build that passed CI (not for production)

## Reverse proxy

Keep immich-edit on loopback or a private Docker network and terminate TLS with a reverse proxy.
Forward `Host`, `X-Forwarded-For`, and `X-Forwarded-Proto`. The backend trusts those headers only
from peers in `TRUSTED_PROXIES` (defaults to loopback and private ranges; see
[Configuration](configuration.md#trusted-proxies)).

### Caddy

```txt
edit.example.com {
  reverse_proxy 127.0.0.1:3000
}
```

### nginx

```nginx
server {
  listen 443 ssl http2;
  server_name edit.example.com;

  ssl_certificate /etc/letsencrypt/live/edit.example.com/fullchain.pem;
  ssl_certificate_key /etc/letsencrypt/live/edit.example.com/privkey.pem;

  client_max_body_size 200m;

  location / {
    proxy_pass http://127.0.0.1:3000;
    proxy_set_header Host $host;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
    proxy_set_header X-Request-Id $request_id;
    proxy_read_timeout 600s;
  }
}
```

Keep `client_max_body_size` above `MAX_BODY_MB`.

## HTTPS for the installed app

Installing immich-edit as a PWA (and running its service worker) requires a trusted HTTPS
certificate — plain `http://` LAN addresses and clicked-through self-signed certs cannot install:

- **Tailscale Serve**: `tailscale serve --bg 3000` provides trusted HTTPS inside your tailnet.
- **Caddy `tls internal`**: install Caddy's root CA on each client device (on iOS/iPadOS also
  enable it in **Settings** > **General** > **About** > **Certificate Trust Settings**).

If you put an auth proxy (such as Cloudflare Access) in front of immich-edit, allow
`/manifest.webmanifest`, `/apple-touch-icon.png`, `/favicon.svg`, `/icon-192.png`,
`/icon-512.png`, and `/icon-maskable-512.png` without login so iOS/iPadOS receives the app icon.

## Health checks

| Endpoint | Auth | Purpose |
| --- | --- | --- |
| `GET /api/health/live` | None | Liveness probe (used by Docker `HEALTHCHECK`) |
| `GET /api/health` | Session | Renderer, Immich, DB, codec, and redacted config state |
| `GET /api/immich/ping` | Session | `204` when Immich responds; `502`/`504` otherwise |

## Native execution

Requires Rust, Node 26, and native image libraries (see `contributing/development.md` in the
repository):

```shell
cd web && npm ci && npm run wasm && npm run build && cd ..
WEB_DIR=web/build DCP_DIR=crates/backend/assets/dcp DATA_DIR=./data \
  cargo run -p immich-edit-backend --release
```
