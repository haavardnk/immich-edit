# Deploying immich-edit

immich-edit talks to Immich over HTTP and serves a static SPA. It does not terminate TLS. For anything beyond a trusted LAN, put it behind a reverse proxy. Users sign in with their own Immich accounts; the Immich connection is configured in the app on first run, not through env vars.

## Modes

### Localhost / native

Good for local testing and macOS GPU work.

```bash
cd web && npm ci && npm run build
cd ..
cargo run -p immich-edit-backend
```

Set `BIND_ADDR=127.0.0.1:3000`, `WEB_DIR=web/build`, and `DCP_DIR=crates/backend/assets/dcp` when running from the repository root.

Open `http://127.0.0.1:3000`.

### Docker on LAN

For a home network where you trust every device that can reach the host.

Use the published Docker Hub image unless you are developing locally:

```yaml
services:
  immich-edit:
    image: haavardnk/immich-edit:latest
    ports:
      - "3000:3000"
    volumes:
      - immich-edit-cache:/cache
    restart: unless-stopped

volumes:
  immich-edit-cache:
```

Run `docker compose up -d` and open `http://<host>:3000`.

On first launch, open the app and complete setup: enter the Immich server URL and sign in with an Immich admin account (password or API key). That admin claims the instance and becomes the first user. Everyone else signs in at `/login` with their own Immich credentials, and edits stay per-user. Complete setup before exposing the instance publicly — the first admin to reach the setup screen claims an unconfigured instance.

For local Docker image builds, clone the repository and use [docker-compose.example.yml](../docker-compose.example.yml). It includes a commented build option for development.

### Image tags

[`haavardnk/immich-edit`](https://hub.docker.com/r/haavardnk/immich-edit) publishes exact semver tags (`0.2.0`, `0.2`, `0`) plus `latest` for the newest stable release and `edge` for the newest build including prereleases. Prereleases update only `edge` and their exact tag, never `latest`.

### Configuration file

immich-edit needs no required settings. Optional infrastructure settings can come from the environment or a TOML file at the path in `IMMICH_EDIT_CONFIG`. Environment variables take precedence over matching file values.

```toml
bind_addr = "127.0.0.1:3000"
cache_dir = "./cache"
renderer = "auto"
```

Start the native backend with `IMMICH_EDIT_CONFIG=/path/to/config.toml`. In Docker, mount the file read-only and set `IMMICH_EDIT_CONFIG` to its container path. The file accepts the lowercase forms of the settings in [.env.example](../.env.example), including `allowed_origins`, cache limits, timeouts, and `debug_endpoints`.

### Reverse-proxy (recommended for TLS)

immich-edit has its own Immich-federated user system, so a separate auth proxy is optional. A reverse proxy is still recommended to terminate TLS (Caddy, Traefik, nginx). Keep immich-edit on loopback or a private Docker network and let the proxy forward to it.

Each user signs in with their own Immich account; there is no shared password to distribute.

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
    image: haavardnk/immich-edit:latest
    networks: [proxy]
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

Check which renderer the running instance picked from the in-app **Settings & diagnostics** page (Server section), which shows `renderer_mode`, `renderer_active`, and `gpu_adapter`. The same data is on `GET /api/health`, which requires a signed-in session; `GET /api/health/live` is an unauthenticated liveness probe.

To force the renderer for testing:

- `IMMICH_EDIT_RENDERER=cpu` - skip GPU even if present
- `IMMICH_EDIT_RENDERER=gpu` - prefer GPU; log an error and fall back to CPU if unavailable
- `IMMICH_EDIT_RENDERER=auto` (default) - pick GPU when available

## Healthchecks

- `GET /api/health/live` - unauthenticated liveness probe used by the Docker `HEALTHCHECK` directive. Returns `{"status":"ok"}` if the process is running.
- `GET /api/health` - authenticated detailed health (`renderer_mode`, `renderer_active`, `gpu_adapter`, structured `immich_status`, DB migration status, redacted config).

Reverse-proxy probes should hit `/api/health/live`. Monitoring that needs detail should authenticate and hit `/api/health`.

## Backups

immich-edit stores everything in `CACHE_DIR` (default `./cache`):

- `immich-edit.db` (sqlite) - edits, edit history, imported-library metadata, and export jobs
- `dcp/` - content-addressed user and bundled DCP files referenced by the database
- `luts/` - imported `.cube` files referenced by edits
- `rasters/` - brush-mask data referenced by edits
- `edited-thumb/` - generated thumbnail cache, safe to delete and rebuild

Back up the database plus `dcp/`, `luts/`, and `rasters/`. Use `sqlite3 cache/immich-edit.db ".backup cache/backup.db"` for a live database snapshot, then copy the referenced directories, or stop the service and copy all of `CACHE_DIR`. Bundled profiles can be restored from the image, but user imports cannot.

Originals stay in Immich; immich-edit never modifies them.

## Upgrades

0.x releases may break compatibility. When upgrading:

1. Stop the service.
2. Back up `cache/immich-edit.db`.
3. Pull the new image or rebuild.
4. Start. Migrations run automatically. If schema changes are incompatible, clear `cache/` and re-edit from scratch.

Check release notes before upgrading across multiple versions.
