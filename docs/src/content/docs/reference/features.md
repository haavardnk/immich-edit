---
title: Features
description: Verified capability matrix for immich-edit.
---

Availability is strictly `✓ Yes` or `✗ No`. `✗ No` reflects absence in this version and is not a
roadmap promise.

## Library, map, and culling

| Capability | Available | Notes |
| --- | :---: | --- |
| Browse timeline, albums, people, tags, folders, favorites, edited, archive, hidden | ✓ Yes | Connected Immich library |
| Album create, rename, describe, cover, remove, delete, leave | ✓ Yes | Photos stay in Immich; owner/editor roles enforced |
| Hierarchical tag tree (create, rename, recolor, sub-tags, delete) | ✓ Yes | Rename requires Immich 3.3+; managed `immich-edit/` tags hidden |
| Smart CLIP search, filename, description, path, and OCR search | ✓ Yes | Smart/OCR require Immich support |
| Multi-faceted filters (rating, label, reject, camera, lens, place, date, size, etc.) | ✓ Yes | Persisted in URL; ranges/multi-pick expand on Immich 3.2+ |
| Saved views and per-view sort order | ✓ Yes | Stored per account |
| Ratings, favorites, tags, color labels, rejects, descriptions | ✓ Yes | Written to Immich after one-time consent; owned photos only |
| Bulk capture time: shift, set by a photo, sync cameras, one time for all, from file names | ✓ Yes | [Metadata](../editing/metadata.md); owned photos |
| Bulk descriptions and copy/paste of description, location, tags, and rating | ✓ Yes | [Metadata](../editing/metadata.md) |
| Title, creator, copyright, camera, lens, and exposure fields | ✗ No | Immich cannot store them |
| Map view of geotagged photos, fullscreen map, and photo info mini map | ✓ Yes | [Map and geotagging](../editing/map.md) |
| Set, move, drag-drop, bulk, and copy/paste photo locations | ✓ Yes | Owned photos; Immich cannot clear GPS in bulk |
| Saved and recent places, coordinates and map links in place search | ✓ Yes | [Map and geotagging](../editing/map.md#saved-and-recent-places) |
| Geotagging from GPX, KML, and TCX tracks with per-camera and per-track time zones and shifts, reference pins, and capture-time correction | ✓ Yes | [Map and geotagging](../editing/map.md#geotag-photos-from-a-track) |
| Two-photo compare, 9-photo survey, face-aware zoom, peek (`W`) | ✓ Yes | Survey is desktop/tablet; phones use compare |
| Virtual copies | ✓ Yes | Independent local edits sharing Immich metadata |
| Move owned photos to Immich trash (with Undo) | ✓ Yes | Refused when Immich trash is disabled |
| Permanent deletion, partner sharing, public links, memories | ✗ No | Use Immich |

## RAW and image input

| Capability | Available | Notes |
| --- | :---: | --- |
| RAW formats decoded by rawler (Bayer 2×2 and X-Trans 6×6) | ✓ Yes | [Supported formats](formats.md) |
| Linear and Bayer DNG with embedded phone/drone corrections | ✓ Yes | ProRAW profile gain table map skipped |
| JPEG, PNG, TIFF, WebP, HEIC, AVIF, JPEG XL, GIF, BMP input | ✓ Yes | Single still frame for animated/multipage files |
| Sigma `.x3f`, PSD/PSB, OpenEXR, HDR/PFM, CMYK/float TIFF | ✗ No | — |
| Pixel-shift / multi-frame RAW merge | ✗ No | Decodes first sub-frame only |

## Develop, detail, and geometry

| Capability | Available | Notes |
| --- | :---: | --- |
| Auto adjust, Auto Tone, Auto WB, WB point/box eyedropper | ✓ Yes | Kelvin + tint for RAW; face-weighted Auto |
| Exposure, tone, presence (texture, clarity, dehaze), vibrance, saturation | ✓ Yes | — |
| RAW clipped-highlight reconstruction and hot/dead pixel repair | ✓ Yes | Automatic before demosaic |
| RGB and Luma curves (with pop-out editor), HSL, Color Grading, B&W mixer | ✓ Yes | [Develop](../editing/develop.md) |
| Targeted on-photo adjustment (`T`) and histogram zone drag | ✓ Yes | — |
| DCP camera profiles and `.cube` 3D LUTs | ✓ Yes | Bundled DCPs + admin imports |
| Custom camera ICC input profiles or `.cube` LUT export | ✗ No | — |
| Automatic lens distortion/vignette profiles and opt-in CA fix | ✓ Yes | Admin-configurable default |
| Capture sharpening, creative sharpening, wavelet luma/color NR | ✓ Yes | CPU and GPU |
| AI Denoise (RAW) and depth-based Lens Blur | ✓ Yes | Requires admin-installed models |
| AI super-resolution, moiré removal, manual defringe | ✗ No | — |
| Crop, custom/saved ratios, straighten, keystone, corner handles, Guided upright | ✓ Yes | [Masks, retouch, and crop](../editing/masks-and-retouch.md) |
| Automatic upright | ✗ No | — |

## Masks and retouch

| Capability | Available | Notes |
| --- | :---: | --- |
| AI masks: Subject, Background, People, Sky, Depth, Scene, Click/Box select | ✓ Yes | Requires admin-installed models |
| Named-person masks, single-instance isolation, text prompts | ✗ No | — |
| Brush, linear, radial, polygon, luminance range, color range masks | ✓ Yes | Add, subtract, intersect; up to 8 layers / 32 shapes |
| Local tone, WB, saturation, vibrance, texture, clarity, sharpening | ✓ Yes | NR, dehaze, HSL, and curves remain global |
| Heal and clone strokes with Visualize dust mode | ✓ Yes | Non-destructive, source-patch based |
| Generative fill, AI object removal, auto dust detection, red-eye | ✗ No | — |

## Workflow and workspace

| Capability | Available | Notes |
| --- | :---: | --- |
| Undo/redo, saved history, snapshots, versions, split/side-by-side compare | ✓ Yes | [Presets, history, and versions](../editing/presets-and-history.md) |
| Presets (0–200% amount, hover preview, JSON import/export) and selective copy/paste | ✓ Yes | — |
| Command palette (`Ctrl+K`), settings search, Quick pins, custom section order | ✓ Yes | [Workspace](../editing/workspace.md) |
| Custom per-account keyboard shortcuts | ✓ Yes | [Keyboard shortcuts](shortcuts.md) |
| Histogram, waveform, RGB parade, vectorscope, floating scope (`S`), Navigator | ✓ Yes | — |
| Secondary-display Viewer window, grey background, clipping (`J`), Soft proof | ✓ Yes | Viewer window is desktop only |
| Desktop, tablet, and phone layouts + installable PWA + Light/Dark/System themes | ✓ Yes | PWA requires trusted HTTPS |
| HDR/panorama/focus merge, tethering, plugins, external-editor handoff | ✗ No | — |

## Export and hosting

| Capability | Available | Notes |
| --- | :---: | --- |
| Full-resolution export up to 65,535 px (JPEG, PNG, TIFF, WebP, HEIC, AVIF, JXL) | ✓ Yes | 16-bit for PNG, TIFF, JXL |
| sRGB and Display P3 output with automatic gamut mapping and safe EXIF | ✓ Yes | No Adobe RGB, Rec.2020, linear, or HDR gain map |
| Resize, output sharpening, watermarks, filename templates, export presets, batch jobs | ✓ Yes | [Export](../editing/export.md) |
| Immich upload, stacking, replace original, conflict policy | ✓ Yes | — |
| DNG/PSD export or XMP sidecar round-trip | ✗ No | — |
| Multi-user Immich auth + Immich OAuth carry-over | ✓ Yes | [OAuth sign-in](../hosting/oauth.md) |
| Linux Vulkan GPU, macOS native Metal, CPU fallback, browser WebGPU previews | ✓ Yes | [Rendering](../hosting/rendering.md) |
