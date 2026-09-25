---
layout: default
title: Deployment
parent: Run the server
nav_order: 1
permalink: /deploy/
---

# Deployment

Use the published Docker image for production. On macOS, run natively to use Metal.

For the Compose file and first-run setup, see [Getting started](getting-started.md). The container
runs as UID and GID `10001`.

## Image tags

[`haavardnk/immich-edit`](https://hub.docker.com/r/haavardnk/immich-edit) publishes:

- Exact semantic versions such as `0.5.1`
- Moving minor and major tags such as `0.5` and `0`
- `latest` for the newest stable release
- `edge` for the newest release, including prereleases

Every release image supports Linux `amd64` and `arm64`.

## Reverse proxy

After first-run setup, keep immich-edit on a private network or loopback address and terminate TLS
with Caddy, nginx, Traefik, or another reverse proxy. Immich authentication is built in; no separate
authentication proxy is required. TLS also turns on browser previews: browsers only offer WebGPU to
HTTPS pages and `localhost`. See
[browser previews on a local network](rendering.md#browser-previews-on-a-local-network).

Forward `Host`, `X-Forwarded-For`, and `X-Forwarded-Proto`. The backend believes forwarding
headers only from the peers in `TRUSTED_PROXIES`, which defaults to loopback and private ranges;
see [trusted proxies](configuration.md#trusted-proxies). `X-Forwarded-Proto: https` makes session
cookies secure.

### Caddy

```caddyfile
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

Keep `client_max_body_size` above `MAX_BODY_MB` when large originals pass through the proxy.

## GPU

The container renders on the CPU until a GPU is passed through. See
[GPU passthrough](gpu-passthrough.md) for AMD, Intel and NVIDIA.

## Health checks

- `GET /api/health/live` is an unauthenticated liveness check and returns success while the process
  is running.
- `GET /api/health` requires a session and reports renderer, Immich, database, codec, and redacted
  configuration state.

The official image includes a Docker health check against `/api/health/live`.

## Native execution

Install the dependencies from [development](development.md), build the frontend, then run:

```shell
cd web
npm ci
npm run build
cd ..
WEB_DIR=web/build \
DCP_DIR=crates/backend/assets/dcp \
DATA_DIR=./data \
cargo run -p immich-edit-backend --release
```

See [configuration](configuration.md) for every setting and
[backup and upgrade](backup-and-upgrade.md) before changing versions.
