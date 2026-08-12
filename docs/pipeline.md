---
layout: default
title: Render pipeline
nav_order: 16
permalink: /pipeline/
---

# Render pipeline

This page defines render-stage and CPU/GPU ownership. Update it when an operator changes stage,
route, resolution, or color-space behavior.

## Operator registry

`crates/raw-pipeline/src/ops.rs` registers operators and sorts them by `(stage, order)`:

```text
Sensor -> WhiteBalance -> Tone -> Color -> Geometry -> Output
```

Every operator implements `Op` and declares one mandatory `GpuRoute`:

| Route | Meaning |
| --- | --- |
| `Fused` | Pointwise Rust and WGSL implementation in the generated process pass |
| `Presence` | Texture, clarity, and luma-pyramid infrastructure |
| `Detail` | Dedicated noise-reduction or capture-sharpen path |
| `Pass(name)` | Dedicated named GPU pass such as retouch or dehaze |
| `Manifest` | Persistence-only entry that never runs as a normal operator |

There is no default route. A new operator that omits it fails to compile. Registry tests check that
named passes exist, manifest operators never become active, and only fused operators expose WGSL
snippets.

Fused math remains written in Rust and WGSL. `src/ops/shader_parity.rs` builds a standalone shader
from each fused operator, runs a scene-linear color grid, and compares it with `apply_cpu` at a
relative tolerance of `1e-4`. This also checks uniform packing.

## CPU flow

`cpu::render_with_cancel`:

1. Demosaics Bayer or X-Trans mosaics.
1. Resolves the camera matrix and DCP baseline.
1. Runs sensor operations, including effective lens corrections.
1. Applies EXIF orientation.
1. Runs WhiteBalance, Tone, Color, and Geometry operators.
1. Runs Output operators.
1. Applies the DCP LookTable and tone curve, destination gamut, and creative 3D LUT.
1. Computes preview metadata and encodes the result.

CPU operators do not skip `Detail` routes. Noise reduction, capture sharpening, creative sharpening,
retouch, dehaze, masks, and geometry all have CPU implementations.

### Sensor-stage cache

`SPATIAL_BOUNDARY = (Tone, -35)` separates expensive source-dependent work from display work.
`CpuRenderer` can cache the lower half for previews: demosaic, lens correction, orientation, white
balance, retouch, noise reduction, capture sharpening, and preview reduction. Exposure, tone,
color, geometry, masks, and output continue from the cached image.

Quality renders, nonstandard preview modes, and masks that change local white balance bypass the
cache. `tests/cpu_cache.rs` compares cache misses and hits with uncached `cpu::render`.

## GPU flow

`GpuRenderer::render_with_cancel` selects a fast path or the presence/detail path. Current order:

| Order | Pass | Responsibility |
| --- | --- | --- |
| 1 | Upload and demosaic | Mosaic or RGB input to scene-linear texture |
| 2 | Sensor | Lens vignette and sensor-space corrections |
| 3 | White-balance preparation | White balance and camera-to-sRGB base |
| 4 | Retouch | Heal and clone strokes when active |
| 5 | Noise reduction | Luma, then color bilateral stages |
| 6 | Capture sharpening | RAW-only deconvolution before preview reduction |
| 7 | Preview reduction | Shared Lanczos3 target when the source is over twice the preview size |
| 8 | Dehaze | Bounded atmosphere estimate and guided filter |
| 9 | Presence | Texture, clarity, and shadows pyramid |
| 10 | Process | Fused operators and geometry sampling |
| 11 | DCP base table | Camera HueSatMap in linear ProPhoto |
| 12 | Masks | Component weight and local adjustment blend |
| 13 | Sharpen | Global and per-pixel masked amount |
| 14 | Effects and output | Vignette, grain, destination gamut, transfer curve |
| 15 | DCP finish | LookTable and profile tone curve |
| 16 | 3D LUT | Display-referred tetrahedral `.cube` sampling |
| 17 | Mask overlay | Optional red coverage overlay |
| 18 | Readback and encode | Warning paint, histogram, and output encoding |

The exact encoder grouping can combine adjacent entries. The order and color-space boundaries are
the contract.

## Resolution and geometry

Both renderers use `geom::preview_ratio` and `geom::resample_target`. A non-quality preview whose
cropped source is at least twice its output size reduces at the spatial boundary. Noise reduction,
retouch, and capture sharpening remain at source resolution; dehaze and later work run near preview
size.

The reduction uses separable Lanczos3 in scene-linear space. Geometry samples fractional source
coordinates with the same Catmull-Rom bicubic kernel on CPU and GPU. The reduction ratio comes from
the active crop, not the full sensor.

### Region of interest

`RenderOptions.roi` is a normalized rectangle in display space after orientation, perspective,
angle, and user crop. The backend composes it into the geometry crop, so lens, masks, and transforms
use the normal path. Vignette and grain remap tile coordinates to the full frame to keep their
appearance stable.

The editor requests the visible region at the required device-pixel size. It draws the tile over the
full preview so overlays and surrounding pixels remain stable while the tile loads. Base, original,
and ROI requests use separate render-queue lanes so concurrent views do not cancel one another.

## Dehaze resolution

Both paths estimate atmosphere after reducing the long edge to at most 256 pixels. When the short
edge is at least 512 pixels, both solve the guided-filter coefficients at quarter resolution,
upsample those coefficients, and evaluate transmission against the full guide. CPU and GPU sampling
rules must change together.

## Camera profiles and tone

RAW rendering has three profile outcomes:

- A matched or explicitly selected DCP supplies camera matrices, optional HueSatMap and LookTable,
  and an optional profile tone curve.
- **Default Color** supplies no tables and uses the built-in tone curve.
- **Flat** skips the profile stage and uses only the camera matrix.

Non-RAW input is always Flat. Neutral RAW rendering carries no hidden content-dependent exposure.
Matched DCP baseline exposure applies only through its explicit profile control.

Working textures use linear scene-referred sRGB primaries except while DCP tables operate in linear
ProPhoto. The profile tone curve precedes output gamut conversion. The creative 3D LUT runs last in
display-referred sRGB.

## Output color and warnings

`RenderOptions.output_color_space` selects sRGB or Display P3. The working space stays unchanged;
the primary matrix, gamut projection, transfer curve, and matching ICC profile apply at output.

Clipping and gamut classes travel through the GPU display texture's alpha channel until readback.
Any pass added after tone must preserve or recompute alpha. CPU and GPU classify the undithered,
post-LUT display result. Warnings are preview-only and never enter edits or exports.

## Masks

Masks are manifest operators with dedicated render paths. Components evaluate in scene space and
stay anchored through lens and geometry changes. Generated masks are stored as ordinary `r8`
rasters, so model inference does not run during preview or export.

CPU builds local layer images and blends them once. GPU computes component weights and blends each
layer through dedicated mask passes. Masked sharpening uses a per-pixel amount map on both paths.

## Add an operator

1. Choose its stage and CPU implementation.
1. Declare its `GpuRoute`.
1. Register it in `default_registry()`.
1. Add manifest round-trip coverage when it persists data.
1. Add fused shader parity or dedicated CPU/GPU parity coverage when it changes pixels.
1. Update this page when ownership, order, resolution, or color space changes.
