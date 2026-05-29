# Deploying immich-edit

immich-edit is a single-user backend that talks to Immich over HTTP and serves a static SPA. It does not terminate TLS. For anything beyond a trusted LAN, put it behind a reverse proxy.

## Modes

### Localhost / native

Good for local testing and macOS GPU work. Binds to loopback by default when `AUTH_TOKEN` is unset.

```bash
cp .env.example .env
$EDITOR .env
cargo run -p immich-edit-backend
```

Open `http://127.0.0.1:3000`.

### Docker on LAN with built-in token auth

For a home network where you trust every device that can reach the host.

1. Copy `.env.example` to `.env` and set `IMMICH_URL`, `IMMICH_API_KEY`, and `AUTH_TOKEN`.
2. Copy `docker-compose.example.yml` to `docker-compose.yml`.
3. `docker compose up -d`.
4. Visit `http://<host>:3000`, paste the token at `/login`.

`AUTH_TOKEN` is required whenever `BIND_ADDR` is not loopback. Setting `IMMICH_EDIT_INSECURE=1` overrides the startup check; only do this if a reverse proxy is fronting the service.

### Reverse-proxy auth (recommended for anything exposed)

Let a reverse proxy handle TLS and authentication (Authelia, Authentik, oauth2-proxy, Caddy `basic_auth`, Traefik ForwardAuth). Keep immich-edit on loopback or a private Docker network.

When the proxy authenticates, you can either:

- Leave `AUTH_TOKEN` set and let proxy users still see the token form. Set `IMMICH_EDIT_INSECURE=1` so the backend does not refuse to start on a non-loopback bind.
- Or unset `AUTH_TOKEN` and bind to loopback (with `IMMICH_EDIT_INSECURE=1` if you bind to `0.0.0.0` for the proxy bridge).

immich-edit has no concept of users itself. The token gate is a single shared secret. Treat it like a SSH key, not a password.

## Reverse-proxy examples

### Caddy

```caddyfile
edit.example.com {
  reverse_proxy 127.0.0.1:3000
}
```

With `basic_auth`. Replace the hash with output from `caddy hash-password`:

```caddyfile
edit.example.com {
  basic_auth {
    you JDJhJDE0J...
  }
  reverse_proxy 127.0.0.1:3000
}
```

### Nginx

```nginx
server {
  listen 443 ssl http2;
  server_name edit.example.com;
  ssl_certificate     /etc/letsencrypt/live/edit.example.com/fullchain.pem;
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

Bump `client_max_body_size` past `MAX_BODY_MB` if you raise the backend limit for very large RAWs.

### Traefik (compose labels)

```yaml
services:
  immich-edit:
    image: ghcr.io/haavardnk/immich-edit:0.1.0-beta.1
    networks: [proxy]
    environment:
      IMMICH_URL: http://immich-server:2283
      IMMICH_API_KEY: ${IMMICH_API_KEY}
      AUTH_TOKEN: ${AUTH_TOKEN}
      IMMICH_EDIT_INSECURE: "1"
    labels:
      - traefik.enable=true
      - traefik.http.routers.immich-edit.rule=Host(`edit.example.com`)
      - traefik.http.routers.immich-edit.entrypoints=websecure
      - traefik.http.routers.immich-edit.tls.certresolver=le
      - traefik.http.services.immich-edit.loadbalancer.server.port=3000
networks:
  proxy:
    external: true
```

## GPU passthrough

GPU rendering uses Vulkan inside the container. The image bundles Mesa Vulkan drivers (AMD, Intel) and the Vulkan loader. NVIDIA needs the host runtime.

| Host | Backend | Compose snippet |
|---|---|---|
| Linux, AMD | Vulkan (radv) | `devices: [/dev/dri:/dev/dri]` + `group_add: [video, render]` |
| Linux, Intel | Vulkan (anv) | `devices: [/dev/dri:/dev/dri]` + `group_add: [video, render]` |
| Linux, NVIDIA | Vulkan (proprietary) | Install [nvidia-container-toolkit](https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/latest/install-guide.html), then `deploy.resources.reservations.devices: [{driver: nvidia, count: 1, capabilities: [gpu]}]` |
| macOS | Metal | Run native (`cargo run`). Metal does not pass into Docker. |

On Unraid (and some other distros without a `render` group), `group_add: [video, render]` fails because the names do not resolve inside the container. Look up the numeric GID of the host group that owns `/dev/dri/renderD128` (`stat -c '%g' /dev/dri/renderD128`) and pass it directly, e.g. `group_add: ["18"]`.

Check which renderer the running instance picked:

```bash
curl -H "Authorization: Bearer $AUTH_TOKEN" http://localhost:3000/api/health \
  | jq '{renderer_mode, renderer_active, gpu_adapter}'
```

To force the renderer for testing:

- `IMMICH_EDIT_RENDERER=cpu` - skip GPU even if present
- `IMMICH_EDIT_RENDERER=gpu` - prefer GPU; log an error and fall back to CPU if unavailable
- `IMMICH_EDIT_RENDERER=auto` (default) - pick GPU when available

## Healthchecks

- `GET /api/health/live` - unauthenticated liveness probe used by the Docker `HEALTHCHECK` directive. Returns `{"status":"ok"}` if the process is running.
- `GET /api/health` - authenticated detailed health (`renderer_mode`, `renderer_active`, `gpu_adapter`, Immich reachability, DB migration status, redacted config).

Reverse-proxy probes should hit `/api/health/live`. Monitoring that needs detail should authenticate and hit `/api/health`.

## Backups

immich-edit stores everything in `CACHE_DIR` (default `./cache`):

- `immich-edit.db` (sqlite) - edits, edit history, export jobs
- `rasters/`, `edited-thumb/` - render cache, safe to drop and rebuild

The sqlite database is the file worth backing up. Use `sqlite3 cache/immich-edit.db ".backup cache/backup.db"`, or stop the service and copy the file.

Originals stay in Immich; immich-edit never modifies them.

## Upgrades

0.x releases may break compatibility. When upgrading:

1. Stop the service.
2. Back up `cache/immich-edit.db`.
3. Pull the new image or rebuild.
4. Start. Migrations run automatically. If schema changes are incompatible, clear `cache/` and re-edit from scratch.

Check release notes before upgrading across multiple versions.
