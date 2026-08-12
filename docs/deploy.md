---
layout: default
title: Deployment
nav_order: 8
permalink: /deploy/
---

# Deployment

Use the published Docker image for production. On macOS, run natively to use Metal.

For the Compose file and first-run setup, see [Getting started](getting-started.md). The container
runs as UID and GID `10001`.

## Image tags

[`haavardnk/immich-edit`](https://hub.docker.com/r/haavardnk/immich-edit) publishes:

- Exact semantic versions such as `0.4.0`
- Moving minor and major tags such as `0.4` and `0`
- `latest` for the newest stable release
- `edge` for the newest release, including prereleases

Every release image supports Linux `amd64` and `arm64`.

## Reverse proxy

After first-run setup, keep immich-edit on a private network or loopback address and terminate TLS
with Caddy, nginx, Traefik, or another reverse proxy. Immich authentication is built in; no separate
authentication proxy is required.

Forward `Host`, `X-Forwarded-For`, and `X-Forwarded-Proto`. The backend trusts forwarding headers
only from loopback and private peers. `X-Forwarded-Proto: https` makes session cookies secure.

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

## GPU passthrough

The image contains Vulkan and Mesa drivers. The host still provides the device and, for NVIDIA,
the vendor runtime.

### AMD or Intel on Linux

Add the DRI device and its groups:

```yaml
services:
  immich-edit:
    devices:
      - /dev/dri:/dev/dri
    group_add:
      - video
      - render
```

On a host without named `video` or `render` groups, use the numeric group ID that owns
`/dev/dri/renderD128`:

```shell
stat -c '%g' /dev/dri/renderD128
```

### NVIDIA on Linux

Install
[NVIDIA Container Toolkit](https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/latest/install-guide.html),
then add:

```yaml
services:
  immich-edit:
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: 1
              capabilities: [gpu]
```

### macOS

Docker cannot pass Metal through its Linux virtual machine. The container uses CPU rendering. Run
the binary natively to use Metal.

Open **Settings** > **Diagnostics** after startup to confirm the active renderer and adapter. Use
`IMMICH_EDIT_RENDERER=cpu` to force CPU or `gpu` to request GPU and log a failure before fallback.

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
