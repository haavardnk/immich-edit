# Troubleshooting

## Backend will not start

### "Data directory not writable"

The configured `DATA_DIR` (or its deprecated `CACHE_DIR` alias) is unwritable. With Docker, ensure the bind-mount or named volume is writable by UID/GID 10001 (the image's non-root user).

### "DATABASE_URL invalid" / sqlite open errors

The default `sqlite://./data/immich-edit.db?mode=rwc` expects `data/` to exist and be writable. If you override `DATABASE_URL`, keep `?mode=rwc` so sqlite creates the file on first start.

### "invalid value for ALLOWED_ORIGINS"

Each CORS origin must be an HTTP or HTTPS origin, not a full URL. Use `https://edit.example.com`, not `https://edit.example.com/api` or `https://edit.example.com/`. Separate env values with commas.

## Sign-in problems

### Stuck on the setup screen

Setup needs an Immich **admin** account for the server you entered. Non-admin credentials are rejected. Confirm the URL is reachable from the immich-edit container and that the account has admin rights in Immich.

### "immich url host is not allowed"

The Immich URL points at a blocked address (link-local, cloud metadata `169.254.169.254`, unspecified, or multicast). Use the server's normal hostname or LAN/private IP. Loopback and private addresses are allowed.

### Cannot log in with a valid Immich account

Password login proxies to Immich; if your Immich uses OAuth-only sign-in, use the **API key** option on the login screen instead. If an admin disabled your access, ask them to re-enable it in Settings. Repeated failures are rate-limited for a few minutes.

## Immich upstream errors

The Settings page shows a specific Immich status. `/api/health` exposes the same data as `immich_status.kind` and `immich_status.message`.

### `unreachable`

The backend cannot reach the configured Immich server. Check that the URL entered during setup is correct, the Immich server is running, and Docker networks let the two containers see each other (typically same Compose network or both on `bridge`). An admin can update the server URL from Settings.

### `api_key_rejected`

Your Immich session is no longer valid. Sign out and sign back in; if you signed in with an API key, generate a new key in Immich (Account Settings > API Keys) and use it.

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

RAW rendering includes hue-preserving recovery for channels clipped near sensor white. It runs with camera white balance on both CPU and GPU paths. It can reduce color casts when one channel clips before the others, but it cannot recover detail when every channel is fully clipped. Bring Highlights down to reveal the recovered transition, then adjust Whites as needed.

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

## AI masks

### A mask type is missing from the New mask menu

An admin must install a model for that mask type under **Settings > Mask models**. Subject, sky, depth, and click-guided masks use separate models. If the whole section is disabled, check `config.segment_runtime` on `GET /api/health`; `off` disables model inference.

### Model installation fails

The server downloads catalog models over HTTPS from GitHub or Hugging Face. Check outbound network access from the container, free space under `DATA_DIR/models/`, and backend logs. The installer rejects an interrupted or changed download when its SHA-256 hash does not match the catalog.

### Mask generation is slow or fails on the GPU

`SEGMENT_RUNTIME=auto` tries WebGPU first and falls back to CPU if the provider cannot start. The backend logs the selected backend and elapsed time after each generation. CPU inference can take several seconds, depending on the model.

`SEGMENT_RUNTIME=gpu` is stricter: it fails the request when WebGPU cannot start. Use `auto` for fallback or `cpu` to skip WebGPU. AI inference uses the same Vulkan or Metal access described in [GPU rendering](#gpu-rendering), but it has its own runtime setting.

The first request after a model is loaded is slower. On a memory-constrained server, keep `SEGMENT_MAX_CONCURRENCY=1` and lower `SEGMENT_IDLE_SECS` so inactive sessions are released sooner.

### A luminance or color range changes after a global edit

Range masks select from the maskless, display-referred image. Global exposure and color edits can therefore change the selection. The local adjustment inside the mask does not feed back into its own selection.

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

`MASK_CACHE_MB` (default 1024) is the largest tunable. Lower it to 256 if running on a small VM. The render cache on disk (`DATA_DIR/rasters`) grows until evicted; it is safe to delete the directory while the service is stopped.

Retained render memory is bounded by three byte budgets that you can lower without touching render quality or speed:

- `RAW_FRAME_CACHE_MB` (default 1024) — decoded preview frames kept in RAM.
- `QUALITY_FRAME_CACHE_MB` (default 512) — decoded full-quality frames kept in RAM.
- `GPU_TEXTURE_CACHE_MB` (default 512) — retained free GPU textures reused between renders.

On a memory-constrained host with a shared GPU (for example Unraid), start with `RENDER_MAX_CONCURRENCY=1`, `RAW_FRAME_CACHE_MB=512`, `QUALITY_FRAME_CACHE_MB=256`, `GPU_TEXTURE_CACHE_MB=256`, and `MASK_CACHE_MB=256`. Only drop `PREVIEW_MAX_EDGE` to 2048 if OOM continues, since that reduces preview resolution.

## Logs and request IDs

Every error response includes a `request_id` that matches the `X-Request-Id` header and the corresponding tracing span. Grep logs for that ID when filing a bug.

Set `RUST_LOG=immich_edit_backend=debug,tower_http=debug` for verbose request logging during diagnosis.
