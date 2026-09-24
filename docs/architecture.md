---
layout: default
title: Architecture
nav_order: 15
permalink: /architecture/
---

# Architecture

immich-edit is a static browser app backed by one Rust process. Immich remains the source of
original assets and shared metadata.

## Workspace

| Path | Responsibility |
| --- | --- |
| `crates/backend` | HTTP, Immich proxy, authentication, SQLite, files, jobs, caches, render dispatch |
| `crates/raw-pipeline` | Decode, process, histogram, and encode; no HTTP, Immich, SQLite, or disk paths |
| `crates/web-render` | The GPU display chain compiled to wasm for the browser, drawing to a canvas |
| `crates/ml` | ONNX model definitions, runtime, tensor preparation, segmentation, and refinement |
| `web` | Static SvelteKit SPA; all product data comes from `/api/*` |

The backend serves `web/build` in production.

## Authentication and users

First-run setup validates an Immich administrator and stores the Immich base URL. Later sign-ins
validate each user's Immich password or API key, or run the Immich OAuth flow with PKCE, and
provision a local user record. The flow state travels in a short-lived encrypted cookie rather than
in server memory, so the code exchange survives a restart but cannot be replayed elsewhere.

The browser receives an opaque HttpOnly session cookie. The backend encrypts each stored Immich
credential with `DATA_DIR/instance.key`. Protected requests receive `AuthCtx`, which carries user
identity, administrator status, server epoch, and a per-user Immich client.

Edits, history, presets, jobs, exports, raster references, and virtual copies are scoped by user.
Favorite, rating, tag, and reject writes go to Immich and are shared.

## Asset identity

Immich assets use UUIDs. Local edit versions use `AssetKey`:

```text
master: <uuid>
copy:   <uuid>_<index>
```

`AssetKey::source()` always returns the Immich UUID. Local tables, previews, masks, history, jobs,
and edited thumbnails use the full key. Decoded original-frame caches use the source UUID so all
copies share one decode.

Copy indices increase monotonically and are never reused. A stale URL or job cannot resolve to a
different copy after deletion.

## Persisted edits

`EditManifest` stores a schema version and a map from stable operator IDs to sparse JSON values.
The registry converts it to and from typed `Edits`. Unknown operator IDs survive round trips where
possible.

Mask rasters are immutable content-addressed files under `DATA_DIR/rasters`. Saving an edit rebuilds
raster references in SQLite so eviction cannot delete a referenced mask. Backup treats the complete
directory as durable.

## Request flow

```text
Browser -> local /api route -> per-user Immich client -> Immich
                    |
                    +-> SQLite and DATA_DIR
                    |
                    +-> render queue -> CPU or GPU renderer -> encoded response
```

The frontend never fetches Immich directly. Backend routes proxy library data and originals, persist
local state, and map Immich failures to typed local errors.

## Preview rendering

Live edits use `POST /api/assets/{key}/preview`. Persisted previews use `GET` with an ETag derived
from edit hash, requested edge, server epoch, profile revision, and warning mode. A matching ETag
returns `304` before entering the render queue.

The queue is latest-wins per asset and lane. Base, untouched original, ROI, source, and source-tile
requests have separate lanes so split view, zoom tiles, and source fetches do not cancel the main
preview.

`POST /api/assets/{key}/source` returns the portable linear source a client renders locally. Its
ETag covers only the sensor-stage edits, the requested edge, the server epoch, the profile
revision, and the `roi` if one is sent, so a display-only change revalidates with `304`. With a
`roi` the server renders the sensor stage for the whole frame at the tile's resolution, then sends
only a window around the region: the header records the window's origin and the full source size,
so the renderer computes texture, clarity, shadows and dehaze radii from the full frame and
samples the window through its position in the frame. The window carries a margin as wide as
those filters reach and starts on the coarsest pyramid level's grid, so a tile renders like the
same region of the full source. The response names the DCP profile it was built with in
`x-source-dcp`. `GET /api/dcp/{id}/raw` and `GET /api/luts/{id}/cube` return the profile and LUT
files with immutable cache headers; mask rasters come from `GET /api/rasters/{id}`.

The viewer measures its visible frame, device-pixel ratio, and source limit. It requests the exact
visible ROI and draws that tile over the stable full-frame preview. Any edit invalidates the tile.

When the browser has WebGPU and the preview renderer setting is **Browser when available**, the
base preview renders in a worker (`web/src/lib/render`). The worker asks the wasm renderer for the
sensor-stage key and the rasters and LUT an edit needs; a changed key fetches a new source while
the last one keeps rendering, so only sensor-stage edits (noise reduction, capture sharpening,
lens, retouch, profile, crop and rotation) reach the server. Frames return as `ImageBitmap`s drawn
into a canvas that replaces the base `<img>`, and their histogram and scopes feed the same panels.
The browser base renders at the fit size. Zoomed past it, the viewer asks for a windowed source for
the visible region and renders that tile locally too, so a slider at 1:1 posts nothing and a pan
posts one tile request. Split view's original still comes from the server. A worker or GPU failure
switches the session to server previews; a failed source request switches that asset.

## Render services and caches

`RenderService` owns one long-lived CPU renderer and an optional GPU renderer. Both cache
source-dependent stages. The service also caches decoded preview and quality frames within byte
budgets.

Other caches include edited thumbnails, click-model embeddings, preview metadata, reusable GPU
textures, and generated mask rasters. Diagnostics reports current use and limits.

GPU device loss marks the renderer unavailable, falls back to CPU, and permits a later rebuild.

## Jobs

Batch apply, copy/paste, reset, download, and Immich upload jobs persist in SQLite. Each job carries
an encrypted copy of the submitting user's Immich credential while active. The credential is
deleted when the job reaches a terminal state.

## Models

The model catalog is compiled into `crates/ml`, but model files are downloaded only after an
administrator requests them. Files are verified, stored under `DATA_DIR/models`, and selected per
model kind. One whole-image session and one click session can remain loaded until the idle timeout.

Generated masks are baked to ordinary rasters. Model inference stays outside the hot preview and
export paths.

## Boundaries

- raw-pipeline does not know about HTTP, Immich, SQLite, or storage locations.
- ml does not know about HTTP, Immich, SQLite, or sidecars.
- Backend orchestration does not implement pixel operations.
- Frontend code does not decode or develop RAW pixels.
- Operator IDs and asset-key syntax are persisted contracts.

See [render pipeline](pipeline.md) for pixel-stage ownership.
