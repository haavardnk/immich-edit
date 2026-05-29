# Troubleshooting

## Backend will not start

### "Refusing to bind to non-loopback without AUTH_TOKEN"

The backend rejects an exposed bind address with no token. Either set `AUTH_TOKEN`, change `BIND_ADDR` to `127.0.0.1:3000`, or set `IMMICH_EDIT_INSECURE=1` if a reverse proxy is fronting the service.

### "Cache directory not writable"

The configured `CACHE_DIR` is unwritable. With Docker, ensure the bind-mount or named volume is writable by UID/GID 10001 (the image's non-root user).

### "DATABASE_URL invalid" / sqlite open errors

The default `sqlite://./cache/immich-edit.db?mode=rwc` expects `cache/` to exist and be writable. If you override `DATABASE_URL`, keep `?mode=rwc` so sqlite creates the file on first start.

### "invalid value for ALLOWED_ORIGINS"

Each CORS origin must be an HTTP or HTTPS origin, not a full URL. Use `https://edit.example.com`, not `https://edit.example.com/api` or `https://edit.example.com/`. Separate env values with commas.

## Immich upstream errors

The Settings page shows a specific Immich status. `/api/health` exposes the same data as `immich_status.kind` and `immich_status.message`.

### `unreachable`

The backend cannot reach `IMMICH_URL`. Check that the URL is correct, the Immich server is running, and Docker networks let the two containers see each other (typically same Compose network or both on `bridge`).

### `api_key_rejected`

`IMMICH_API_KEY` is rejected. Generate a new key in Immich (Account Settings > API Keys) and restart.

### `timeout`

A request to Immich took longer than the configured timeout. For large RAW downloads, raise `ORIGINAL_TIMEOUT_SECS` (default 120). For export/upload, raise `EXPORT_TIMEOUT_SECS` (default 300).

### `upstream_5xx`

The backend retries idempotent GETs with jittered exponential backoff. Persistent failures usually indicate Immich is restarting, overloaded, or behind a reverse proxy with a strict idle timeout. Check Immich logs first.

## RAW decoding

### "Unsupported format"

The decoder relies on [rawler](https://github.com/dnglab/dnglab/tree/main/rawler). If a specific camera body is missing, file an issue with the file extension, camera model, and a sample file if possible. As a workaround, convert the file to DNG with Adobe DNG Converter.

### Demosaiced colors look wrong

Check that camera white balance is selected, not auto. If colors are still off, the camera's color matrix may be missing in rawler. Try setting custom temp/tint manually.

### Highlights blow out hard

The pipeline has basic clip recovery only. Bring Highlights down and lift Whites. Full highlight reconstruction is not implemented yet.

## GPU rendering

### Renderer shows CPU even though I have a GPU

Hit `GET /api/health` (authenticated) and check `renderer_active` and `gpu_adapter`. If `gpu_adapter` is empty:

- **Docker, AMD/Intel**: confirm `/dev/dri` is passed in and `video` + `render` groups are added. Run `vulkaninfo --summary` inside the container to verify the loader sees the GPU.
- **Docker, NVIDIA**: confirm `nvidia-container-toolkit` is installed on the host and the compose `deploy.resources.reservations.devices` block is present. `nvidia-smi` inside the container should list the GPU.
- **macOS native**: should always pick Metal. If not, check the wgpu adapter line in the startup logs.
- **macOS in Docker**: Metal cannot pass through to a container. Use native execution or accept the CPU fallback.

### "Device lost" mid-render

GPU drivers occasionally drop the wgpu device under memory pressure or after a driver crash. The backend logs the event and falls back to CPU for in-flight renders. Restart the service to recreate the GPU device. If it keeps happening, lower `MASK_CACHE_MB` and `RENDER_MAX_CONCURRENCY`.

### Renders are slow on small RAWs

The first render warms up GPU pipelines and uploads textures; expect a delay of 1 to 3 seconds. Subsequent edits to the same asset should be sub-second.

## Edits and history

### "409 Conflict" when saving

Two browser tabs edited the same asset. The losing tab gets the current server state in the 409 response. Discard or merge changes manually; there is no automatic merge.

### Export uploads to Immich but tags/albums fail

The upload is recorded as complete; tag/album/stack steps are best-effort. The result panel surfaces warnings inline. Retrying the same export uses the same idempotency key and skips the upload, only retrying the side-effect steps.

## Performance

### How do I read `/api/debug/timings`?

Set `IMMICH_EDIT_DEBUG=1` and authenticate. The endpoint returns recent render latency buckets for CPU and GPU (`count`, `p50_us`, `p95_us`, `p99_us`, `max_us`) plus GPU pool memory when the GPU renderer is active.

Use it to compare CPU and GPU runs, spot slow tail latency, and see whether GPU texture pools are growing. The Settings page shows the same data.

### CPU rendering is slow

Use the GPU if at all possible. CPU demosaic + tone on a 24MP RAW takes several seconds. The CPU path is correctness-first, not throughput-first.

### Memory keeps growing

`MASK_CACHE_MB` (default 1024) is the largest tunable. Lower it to 256 if running on a small VM. The render cache on disk (`CACHE_DIR/rasters`) grows until evicted; it is safe to delete the directory while the service is stopped.

## Logs and request IDs

Every error response includes a `request_id` that matches the `X-Request-Id` header and the corresponding tracing span. Grep logs for that ID when filing a bug.

Set `RUST_LOG=immich_edit_backend=debug,tower_http=debug` for verbose request logging during diagnosis.
