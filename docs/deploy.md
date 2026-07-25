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
      - immich-edit-data:/data
    restart: unless-stopped

volumes:
  immich-edit-data:
```

Run `docker compose up -d` and open `http://<host>:3000`.

On first launch, open the app and complete setup: enter the Immich server URL and sign in with an Immich admin account (password or API key). That admin claims the instance and becomes the first user. Everyone else signs in at `/login` with their own Immich credentials, and edits stay per-user. Complete setup before exposing the instance publicly — the first admin to reach the setup screen claims an unconfigured instance.

For local Docker image builds, clone the repository and use [docker-compose.example.yml](../docker-compose.example.yml). It includes a commented build option for development.

### Image tags

[`haavardnk/immich-edit`](https://hub.docker.com/r/haavardnk/immich-edit) publishes exact semver tags (`0.3.0`, `0.3`, `0`) plus `latest` for the newest stable release and `edge` for the newest build including prereleases. Prereleases update only `edge` and their exact tag, never `latest`.

### Configuration file

immich-edit needs no required settings. Optional infrastructure settings can come from the environment or a TOML file at the path in `IMMICH_EDIT_CONFIG`. Environment variables take precedence over matching file values.

```toml
bind_addr = "127.0.0.1:3000"
data_dir = "./data"
renderer = "auto"
segment_runtime = "auto"
```

Start the native backend with `IMMICH_EDIT_CONFIG=/path/to/config.toml`. In Docker, mount the file read-only and set `IMMICH_EDIT_CONFIG` to its container path. The file accepts the lowercase forms of the settings in [.env.example](../.env.example), including `allowed_origins`, cache limits, timeouts, and `debug_endpoints`.

### Reverse-proxy (recommended for TLS)

immich-edit has its own Immich-federated user system, so a separate auth proxy is optional. A reverse proxy is still recommended to terminate TLS (Caddy, Traefik, nginx). Keep immich-edit on loopback or a private Docker network and let the proxy forward to it.

Each user signs in with their own Immich account; there is no shared password to distribute.

When the proxy connects from a loopback or private address, immich-edit automatically honors its `X-Forwarded-For` client IP (for rate limiting and session records) and `X-Forwarded-Proto` scheme (to mark the session cookie `Secure` over HTTPS). Forwarding headers from public direct connections are ignored, so no configuration is needed.

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

GPU rendering and AI mask inference use Vulkan inside the container. The image bundles Mesa Vulkan drivers (AMD, Intel) and the Vulkan loader. NVIDIA needs the host runtime.

| Host | Backend | Compose snippet |
| --- | --- | --- |
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

## AI mask models

AI masks run on the server. No model is bundled with the image. An admin chooses models under **Settings > Mask models**, where the app shows download size, licence, and estimated CPU and GPU cost. Models are shared by all users and stored under `DATA_DIR/models/`.

Installing a model requires outbound HTTPS to its catalog source. The current catalog uses GitHub and Hugging Face. Every download is checked against a pinned SHA-256 hash before it is installed.

Segmentation has separate runtime settings from the image renderer:

- `SEGMENT_RUNTIME=auto` (default) - try WebGPU, then fall back to CPU if it cannot start
- `SEGMENT_RUNTIME=gpu` - require WebGPU; mask generation fails instead of falling back
- `SEGMENT_RUNTIME=cpu` - run inference on CPU
- `SEGMENT_RUNTIME=off` - disable AI masks
- `SEGMENT_MAX_EDGE=2048` - cap the source image sent to inference; valid range is 256 to 8192
- `SEGMENT_MAX_CONCURRENCY=1` - limit concurrent inference jobs
- `SEGMENT_IDLE_SECS=60` - unload inactive model sessions after this many seconds

The first generation after installing a model or after its session unloads takes longer because the runtime has to load the model again. Existing generated masks are ordinary local rasters and continue to render if their model is removed.

## Healthchecks

- `GET /api/health/live` - unauthenticated liveness probe used by the Docker `HEALTHCHECK` directive. Returns `{"status":"ok"}` if the process is running.
- `GET /api/health` - authenticated detailed health (`renderer_mode`, `renderer_active`, `gpu_adapter`, structured `immich_status`, DB migration status, redacted config).

Reverse-proxy probes should hit `/api/health/live`. Monitoring that needs detail should authenticate and hit `/api/health`.

## Backups

immich-edit keeps durable data in `DATA_DIR` (default `./data`, `/data` in Docker):

- `immich-edit.db` (sqlite) - edits, edit history, imported-library metadata, and export jobs
- `instance.key` - AES key that decrypts each user's stored Immich credentials; without it the database's saved logins are unrecoverable
- `dcp/` - content-addressed user and bundled DCP files referenced by the database
- `luts/` - imported `.cube` files referenced by edits
- `models/` - downloaded AI mask models; reinstallable from Settings
- `rasters/` - brush-mask data referenced by edits
- `cache/` - regenerable thumbnail, embedding, and export staging caches; safe to delete

Back up the whole of `DATA_DIR` except `cache/`. Use `sqlite3 data/immich-edit.db ".backup data/backup.db"` for a live database snapshot, then copy `instance.key` and the referenced directories, or stop the service and copy all of `DATA_DIR`. Keep `instance.key` with the database; a database restored without its key cannot decrypt stored credentials. Bundled profiles and catalog models can be restored, but user imports and generated mask rasters cannot.

Originals stay in Immich; immich-edit never modifies them.

## Upgrades

0.x releases may break compatibility. When upgrading:

1. Stop the service.
2. Back up `data/immich-edit.db` and `data/instance.key`.
3. Pull the new image or rebuild.
4. Start. Migrations run automatically. If schema changes are incompatible, clear `DATA_DIR` and re-edit from scratch.

Check release notes before upgrading across multiple versions.

### Upgrading from v0.2.x to v0.3.0

v0.3.0 replaces the shared Immich URL and API key with in-app setup and per-user Immich sign-in. `IMMICH_URL`, `IMMICH_API_KEY`, and `AUTH_TOKEN` are no longer read. Remove them, start the new version, and complete `/setup` with an Immich admin account.

Database migrations preserve old edits, history, presets, and jobs under a legacy owner, but they are not assigned to the new accounts and do not appear after sign-in. There is no supported reassignment tool. Keep the pre-upgrade backup if those records matter.

### Directory layout changes in v0.3.0

Earlier releases stored everything under `CACHE_DIR` (`/cache` in Docker). 0.3.0 renames this to `DATA_DIR` and moves regenerable caches into a `cache/` subdirectory. `CACHE_DIR` still works as a deprecated alias, so native setups need no change; the backend logs a warning until you rename it to `DATA_DIR`.

Docker users must remount their existing volume from `/cache` to `/data` (the data sits at the volume root, so only the target path changes):

```yaml
    volumes:
      - immich-edit-cache:/data
```

Keep the old volume name if you want; only the mount target matters. The stale `edited-thumb/` and `embeddings/` directories left at the volume root are harmless and can be deleted.
