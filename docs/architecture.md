---
layout: default
title: Architecture
parent: Contribute
nav_order: 2
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
| `crates/ml` | ONNX model definitions, runtime, tensor preparation, segmentation, refinement, and tiled denoising |
| `crates/raw-pipeline-testkit` | Test-only fixtures, synthetic frames, parity metrics, and GPU setup shared by raw-pipeline tests |
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
   |                |
   |                +-> SQLite and DATA_DIR
   |                |
   |                +-> render queue -> CPU or GPU renderer -> encoded image or linear source
   |
   +-> render worker -> wasm display chain -> canvas
```

The frontend never fetches Immich directly. Backend routes proxy library data and originals, persist
local state, and map Immich failures to typed local errors. [Rendering](rendering.md) describes the
server and browser split for operators; the sections below describe its implementation.

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
`roi` the source is a window around the region: the header records the window's origin and the
full source size, and the renderer samples the window through its position in the frame. A GPU
server pads the window by its local filter reach only and attaches a context section, the whole
frame's dehaze grid and blurred texture, clarity and shadows levels cropped to the window, so a
tile renders byte for byte like the server's render of the region; the `roi` ETag also covers
`source::context_key`. When the frame does not fit the GPU, the server builds the context section
by section and runs the sensor stage for the window only. A CPU server pads the window as far as
dehaze and presence reach and sends no context. The response names the DCP profile it was built
with in `x-source-dcp`. `GET /api/dcp/{id}/raw` and `GET /api/luts/{id}/cube` return the profile
and LUT files with immutable cache headers; mask rasters come from `GET /api/rasters/{id}`.

The viewer measures its visible frame, device-pixel ratio, and source limit. It requests the exact
visible ROI and draws that tile over the stable full-frame preview. Any edit invalidates the tile.

When the browser has WebGPU and the preview renderer setting picks it (**Browser**, or **Fastest**
when the browser measured clearly quicker; see [rendering](rendering.md#the-browser)), the
base preview renders in a worker (`web/src/lib/render`). The worker asks the wasm renderer for the
sensor-stage key and the rasters and LUT an edit needs; a changed key fetches a new source while
the last one keeps rendering, so only sensor-stage edits (noise reduction, capture sharpening,
lens, retouch, profile, crop and rotation) reach the server. Frames return as `ImageBitmap`s drawn
into a canvas that replaces the base `<img>`, and their histogram and scopes feed the same panels.
The browser base renders at the fit size. Zoomed past it, the viewer asks for a windowed source for
the visible region and renders that tile locally too, so a pan posts one tile request. A tile with
a context section is also keyed on the wasm renderer's context key; a changed key fetches a new
tile while the loaded one keeps drawing. Split view's original still comes from the server. A
worker or GPU failure switches the session to server previews; a failed source request switches
that asset.
Each `/preview` response carries a `Server-Timing` header with `render` and `encode` durations,
which **Fastest** adds to the download and decode time it measures to time the server.

## Render services and caches

`RenderService` owns one long-lived CPU renderer and an optional GPU renderer. Both cache
source-dependent stages and admit renders against one shared host memory budget, evicting idle
cached stages of either renderer first; the GPU renderer also admits against its GPU memory budget.
The service also caches decoded preview and quality frames within byte budgets and refuses to
decode a frame larger than the host budget.

Other caches include edited thumbnails, click-model embeddings, AI Denoise results, preview
metadata, reusable GPU textures, and generated mask rasters. Diagnostics reports current use and
limits.

GPU device loss marks the renderer unavailable, falls back to CPU, and permits a later rebuild. A
GPU error, a panic, or a frame past the device limits renders that request on the CPU and keeps the
GPU renderer.

## Jobs

Batch apply, copy/paste, reset, download, and Immich upload jobs persist in SQLite. Each job carries
an encrypted copy of the submitting user's Immich credential while active. The credential is
deleted when the job reaches a terminal state.

## Models

The model catalog is compiled into `crates/ml`, but model files are downloaded only after an
administrator requests them. Files are verified, stored under `DATA_DIR/models`, and selected per
model kind. One whole-image session, one click session, and one denoise session can remain loaded
until the idle timeout. Every inference job shares the `ML_MAX_CONCURRENCY` limit.

Generated masks are baked to ordinary rasters. Model inference stays outside the hot preview and
export paths.

AI Denoise runs the model once per RAW photo and user, in tiles over the full-resolution mosaic, and
stores the result under `DATA_DIR/cache/denoise`. Renders blend that stored result into the sensor
stage at the chosen strength; an export waits for a pending result first.

## Boundaries

- raw-pipeline does not know about HTTP, Immich, SQLite, or storage locations.
- ml does not know about HTTP, Immich, SQLite, or sidecars.
- Backend orchestration does not implement pixel operations.
- Frontend code does not decode or develop RAW pixels.
- Operator IDs and asset-key syntax are persisted contracts.

See [render pipeline](pipeline.md) for pixel-stage ownership.
