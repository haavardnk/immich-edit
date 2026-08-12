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
| `crates/ml` | ONNX model definitions, runtime, tensor preparation, segmentation, and refinement |
| `web` | Static SvelteKit SPA; all product data comes from `/api/*` |

The backend serves `web/build` in production.

## Authentication and users

First-run setup validates an Immich administrator and stores the Immich base URL. Later sign-ins
validate each user's Immich password or API key and provision a local user record.

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

The queue is latest-wins per asset and lane. Base, untouched original, and ROI requests have
separate lanes so split view and zoom tiles do not cancel the main preview.

The viewer measures its visible frame, device-pixel ratio, and source limit. It requests the exact
visible ROI and draws that tile over the stable full-frame preview. Any edit invalidates the tile.

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
