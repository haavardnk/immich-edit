# immich-edit

A non-destructive RAW editor for your [Immich](https://immich.app/) library. Browse albums in the browser, render previews and exports on the server, and keep the edits outside Immich. Originals stay untouched.

> **Beta software, early development.** Expect bugs, missing features, breaking changes, and occasional migrations that require clearing the cache or database. There is no upgrade path between 0.x releases yet. Run it against a backup, not your only copy.

## Why

I wanted Lightroom-style edits on my Immich library without sending photos to the cloud, without exporting to disk first, and without giving up RAW. Nothing in the Immich plugin ecosystem covered this, so I started building it. It is opinionated, single-user, and shaped around how I work.

## What works

Rendering and decoding:

- All RAW formats that [rawler](https://github.com/dnglab/dnglab/tree/main/rawler) supports (.arw, .cr2, .cr3, .nef, .dng, etc.)
- JPEG, PNG, TIFF, WebP, HEIC, AVIF, JPEG XL, GIF, BMP
- GPU rendering via wgpu (Vulkan on Linux, Metal on macOS), CPU fallback always available

Edits:

- Exposure, contrast, brightness, highlights, shadows, blacks, whites
- White balance (camera, auto, custom temp/tint)
- HSL, saturation, vibrance, color grading
- Curves (RGB, R, G, B, luma)
- Clarity, texture, dehaze, sharpening, luma + color noise reduction
- Vignette, grain
- Crop, rotate, flip
- Local masks (radial, linear, brush) with adjustable parameters
- Lens corrections via lensfun profiles (distortion, vignette, chromatic aberration)

Export:

- JPEG, PNG (8/16-bit), WebP, AVIF, HEIC, TIFF (8/16-bit), JPEG XL (8/16-bit)
- Push edited results back to Immich as a new asset

## What does not work yet

- Single-user only. One shared auth token, no accounts.
- No HDR output, no DNG export, no PSD compatibility, no LUT support
- No AI features
- No presets
- Histograms and clipping warnings are basic
- No mobile layout
- No undo history beyond the current session
- CPU rendering is slow; use the GPU path if you can

## Quick start

The Docker Hub image is not published yet. For now, Docker Compose builds the image locally.

```bash
git clone https://github.com/haavardnk/immich-edit.git
cd immich-edit
cp .env.example .env
cp docker-compose.example.yml docker-compose.yml
# edit .env
docker compose up -d
```

Open `http://localhost:3000` and log in with the token.

`AUTH_TOKEN` is optional only when the server binds to a loopback address. The Docker Compose example binds to `0.0.0.0`, so set a token unless you also change the bind/security settings.

For anything beyond a trusted LAN, put immich-edit behind a reverse proxy that handles TLS and authentication (Authelia, Authentik, oauth2-proxy, Caddy `basic_auth`, Traefik ForwardAuth). The shared token is a single secret, not a user system. See [docs/deploy.md](docs/deploy.md) for proxy examples.

## Documentation

- [Deploy guide](docs/deploy.md) - Docker, native, reverse-proxy, GPU passthrough, backups, upgrades
- [Troubleshooting](docs/troubleshooting.md) - common errors and how to diagnose them
- [Raw pipeline](docs/pipeline.md) - contributor reference for operator and render-pass ownership

## Configuration

Settings use environment variables. See [.env.example](.env.example) for the full list.

`IMMICH_URL` and `IMMICH_API_KEY` are required. Most other settings can stay unset.

## GPU acceleration

GPU rendering is much faster than CPU rendering, especially on large RAWs. `wgpu` picks the backend at startup. Check `GET /api/health` to see which renderer is active.

To enable a GPU in Docker, uncomment the matching block in [docker-compose.example.yml](docker-compose.example.yml) and restart.

| Host | Backend | Setup |
|---|---|---|
| Linux, AMD or Intel iGPU | Vulkan | Pass `/dev/dri` and add `video` + `render` groups. The image includes Mesa Vulkan drivers. |
| Linux, NVIDIA | Vulkan | Install `nvidia-container-toolkit` on the host and use the `deploy.resources.reservations.devices` block. |
| macOS, native | Metal | Run the binary directly. |
| macOS, in Docker | none | Falls back to CPU. Metal cannot be passed into a container. |

`IMMICH_EDIT_RENDERER` controls the renderer:

- `auto` (default): use GPU when available, otherwise use CPU
- `gpu`: prefer GPU and log an error if it is missing, then use CPU
- `cpu`: use CPU only

If the GPU path is not active, check the backend startup logs for the wgpu adapter line.

## Development

Local development runs without Docker. The backend uses `cargo run`; the frontend uses Vite. See [CONTRIBUTING.md](CONTRIBUTING.md) for system dependencies and commands.

Tests:

```bash
cargo test --workspace
cd web && npm run check
```

## License

[AGPL-3.0-only](LICENSE).

Use it, modify it, run it on your own server. If you host a modified version where other people can reach it over a network, you have to make your source available to those users.

## Acknowledgments

- [Immich](https://immich.app/) for the platform this plugs into
- [RapidRAW](https://github.com/CyberTimon/RapidRAW) for pipeline inspiration
- [rawler](https://github.com/dnglab/dnglab) for RAW parsing
- [wgpu](https://wgpu.rs/) for GPU rendering in Rust
- [lensfun](https://lensfun.github.io/) for the lens correction database
