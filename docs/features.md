---
layout: default
title: Features
nav_order: 2
permalink: /features/
---

# Features

This matrix covers v0.4.0. `✗ No` does not imply planned support.

## Immich library and culling

| Capability | Available | Notes |
| --- | :---: | --- |
| Browse timeline, archive, hidden assets, albums, favorites, folders, people, and tags | ✓ Yes | Uses the connected Immich library |
| Read-only album browsing | ✓ Yes | Albums and their assets come from Immich |
| Contextual smart search | ✓ Yes | Uses Immich CLIP search |
| Filename search | ✓ Yes | Searches Immich metadata |
| Rating, favorite, date, filename, visibility, and reject filters | ✓ Yes | Available in browse and search views; changing a filter clears the current selection |
| Thumbnail size controls | ✓ Yes | Four remembered sizes remain available on narrow screens |
| Per-view sort order | ✓ Yes | Timelines open newest first, albums, people, tags, and folders oldest first, and each remembers the direction you pick |
| Camera, lens, location, OCR, multi-person, or album-membership search filters | ✗ No | Use Immich for advanced metadata search |
| Ratings, favorites, tags, and reject marks | ✓ Yes | Writes metadata to Immich after consent |
| Edit descriptions, dates, locations, or EXIF fields | ✗ No | Metadata writes are limited to ratings, favorites, and tags |
| Multi-select and batch actions | ✓ Yes | Selecting every filtered result enables job actions; individual photo selections enable metadata, culling, tags, and virtual copies |
| Two-photo compare | ✓ Yes | Synchronized or independent zoom and pan |
| Fullscreen viewing | ✓ Yes | `Shift+F` hides the chrome in loupe, compare, survey, and editor views |
| Survey view | ✓ Yes | Up to nine photos |
| Virtual copies | ✓ Yes | Independent local edits without duplicating the original |
| Color labels | ✗ No | — |
| Create, rename, delete, or share Immich albums | ✗ No | Existing albums can receive exported edits |
| Stack an exported edit with its original | ✓ Yes | Either asset can be the stack primary |
| Browse, create, or reorganize arbitrary Immich stacks | ✗ No | Export stacking is the only stack operation |
| Archive, trash, restore, or delete assets | ✗ No | Reject uses an `immich-edit/reject` tag and never deletes |
| Map and location browsing | ✗ No | — |
| Immich memories | ✗ No | — |
| Partner sharing and public share links | ✗ No | Use Immich for sharing |

## Input and RAW compatibility

| Capability | Available | Notes |
| --- | :---: | --- |
| RAW formats supported by rawler | ✓ Yes | [Camera and format details](compatibility.md) |
| Bayer RAW | ✓ Yes | Native 2x2 CPU and GPU demosaic |
| Fujifilm X-Trans RAW | ✓ Yes | Native 6x6 CPU and GPU demosaic |
| Linear and Bayer DNG input | ✓ Yes | Tested end to end |
| JPEG, PNG, TIFF, WebP, HEIC, AVIF, JPEG XL, GIF, and BMP input | ✓ Yes | [Format details](compatibility.md) |
| Sigma Foveon `.x3f` | ✗ No | Not decoded by rawler |
| CFA patterns other than 2x2 Bayer or 6x6 X-Trans | ✗ No | Rejected rather than rendered incorrectly |
| Pixel-shift and multi-frame RAW merge | ✗ No | Only the first sub-frame is decoded |
| Animated or multipage image sequences | ✗ No | GIF, TIFF, HEIF, and similar containers decode one still image |
| Embedded-JPEG fallback for rejected RAW files | ✗ No | RAW decode must succeed |
| Floating-point, CMYK, or unsupported tiled TIFF | ✗ No | Integer RGB, RGBA, and grayscale only |
| PSD and PSB input | ✗ No | — |
| OpenEXR, Radiance HDR, and PFM input | ✗ No | — |

## RAW development

| Capability | Available | Notes |
| --- | :---: | --- |
| Auto adjust | ✓ Yes | Reads Bayer and X-Trans mosaics |
| Camera and custom white balance | ✓ Yes | Temperature and tint controls |
| Exposure and tone controls | ✓ Yes | Exposure, brightness, contrast, highlights, shadows, whites, and blacks |
| RAW clipped-highlight reconstruction | ✓ Yes | Reconstructs clipped channels before tone work |
| Sensor black- and white-point overrides | ✗ No | No RAW preprocessing controls |
| RGB and luma curves | ✓ Yes | Composite and per-channel curves |
| HSL and color grading | ✓ Yes | HSL mixer and color wheels |
| Black-and-white channel mixer | ✗ No | Saturation can be reduced, but there is no dedicated mixer |
| Channel mixer or calibration-primary controls | ✗ No | — |
| DCP camera profiles | ✓ Yes | Auto, Default Color, Flat, bundled profiles, and admin imports |
| Custom camera ICC input profiles | ✗ No | Camera profiles use DCP only |
| Creative 3D LUTs | ✓ Yes | `.cube` import with amount control; importing is admin-only |
| 3D LUT export | ✗ No | No `.cube` download or baking from current color edits |
| Automatic RAW lens profiles | ✓ Yes | Distortion and vignette; chromatic aberration remains opt-in |
| Crop, rotate, and flip | ✓ Yes | Geometry controls |
| Manual perspective correction | ✓ Yes | Keystone, aspect, and corner controls |
| Automatic or guided upright | ✗ No | — |

## Detail

| Capability | Available | Notes |
| --- | :---: | --- |
| Capture sharpening | ✓ Yes | RAW-only default, switchable per photo |
| Creative sharpening | ✓ Yes | Amount, radius, detail, and masking |
| Export-stage output sharpening | ✗ No | No screen, matte, or glossy pass separate from Develop sharpening |
| Deconvolution sharpening | ✗ No | Unsharp-mask style sharpening only |
| Luma and color noise reduction | ✓ Yes | CPU and GPU |
| AI denoise | ✗ No | — |
| AI super resolution or upscale | ✗ No | — |
| Texture, clarity, and dehaze | ✓ Yes | Global controls; texture and clarity also work in masks |
| Hot or dead pixel correction | ✗ No | — |
| Demosaic method selection or dual demosaic | ✗ No | One method per CFA type |
| Alternate high-quality export demosaic | ✗ No | Exports use the same Bayer or X-Trans method as previews |
| Moiré and false-color correction | ✗ No | — |
| Manual purple or green defringe | ✗ No | Lens chromatic-aberration correction is profile based |
| Depth-based lens blur and bokeh | ✗ No | Depth masks exist, but there is no blur operator |

## Masks and retouch

| Capability | Available | Notes |
| --- | :---: | --- |
| Linear, radial, brush, and polygon masks | ✓ Yes | Non-destructive local adjustments |
| Luminance and color range masks | ✓ Yes | Eyedropper and softness controls |
| Subject, background, people, sky, depth, and scene masks | ✓ Yes | Requires admin-installed models |
| Named-person or face-aware selection | ✗ No | Immich people data is not used for masks |
| Select one person or object from several instances | ✗ No | Whole-class masks merge matching instances |
| Text-prompt masks | ✗ No | — |
| Click and box object selection | ✓ Yes | Positive and negative point refinement |
| Add, subtract, and intersect components | ✓ Yes | Components combine inside a mask layer |
| Local tone and color edits | ✓ Yes | Exposure, tone, white balance, saturation, and vibrance |
| Local texture, clarity, and sharpening | ✓ Yes | Per-mask deltas |
| Local noise reduction, dehaze, HSL, or curves | ✗ No | These controls remain global |
| Copy masks between photos | ✓ Yes | Opt in with **Masks** when copying edits |
| Copy heal and clone strokes | ✓ Yes | Opt in with **Retouch** when copying edits |
| Heal and clone | ✓ Yes | Parametric, non-destructive strokes |
| Content-aware or AI object removal | ✗ No | Heal and clone require a source patch |
| Generative fill or expand | ✗ No | No cloud or local generative model |
| Automatic dust, spot, or distraction detection | ✗ No | Retouch strokes are placed manually |
| Red-eye correction | ✗ No | — |

## Workflow and compositing

| Capability | Available | Notes |
| --- | :---: | --- |
| Undo, redo, and saved history | ✓ Yes | History entries can be restored |
| Presets | ✓ Yes | Save, apply, and batch apply |
| Batch editing and export | ✓ Yes | Persistent background jobs |
| Before and after view | ✓ Yes | Split view and hold-original control |
| True 1:1 viewing | ✓ Yes | Source-resolution server-rendered tiles |
| Editor and loupe filmstrips | ✓ Yes | Each view keeps its own visible or hidden state |
| Clipping and gamut warnings | ✓ Yes | Red, blue, and magenta overlays |
| Waveform, RGB parade, or vectorscope | ✗ No | Histogram only |
| HDR or panorama merge | ✗ No | No multi-frame compositing |
| Focus-stack merge | ✗ No | — |
| Tethered capture | ✗ No | The library comes from Immich |
| Plugin or scripting API | ✗ No | — |
| External-editor handoff | ✗ No | Export and open the file manually |

## Export, color, and interoperability

| Capability | Available | Notes |
| --- | :---: | --- |
| Full-resolution export | ✓ Yes | Uses the quality render path up to 65,535 pixels per edge |
| JPEG, PNG, WebP, AVIF, HEIC, TIFF, and JPEG XL export | ✓ Yes | Format options vary by bit depth and color space |
| 16-bit export | ✓ Yes | PNG, TIFF, and JPEG XL |
| Linear 16-bit or floating-point export | ✗ No | 16-bit exports include the display tone curve |
| Lossless WebP export | ✓ Yes | Required automatically when preserving EXIF in WebP |
| sRGB and Display P3 output | ✓ Yes | ICC profile embedded where the format supports it |
| Adobe RGB, ProPhoto RGB, Rec.2020, or custom ICC output | ✗ No | — |
| Automatic output-gamut projection | ✓ Yes | Pulls out-of-gamut colors toward neutral |
| sRGB and Display P3 soft proof | ✓ Yes | Optional gamut warning overlay |
| Printer-profile soft proof and rendering intents | ✗ No | Soft proof is limited to sRGB and Display P3 |
| Safe EXIF preservation | ✓ Yes | Optional; embedded previews and image-strip tags are removed |
| Selective metadata or GPS stripping | ✗ No | EXIF is all or none |
| Export resize by dimensions, edge, or megapixels | ✗ No | Exports use the edited image's full resolution |
| Filename templates and sequence numbering | ✗ No | Immich uploads use one suffix plus collision numbering |
| Export presets or recipes | ✗ No | Edit presets do not store export settings |
| JPEG chroma-subsampling control | ✗ No | Export uses 4:2:0 |
| Export watermark | ✗ No | — |
| Export to Immich | ✓ Yes | Can assign albums, tags, favorite state, and stack with the original |
| DNG export | ✗ No | — |
| PSD export | ✗ No | — |
| XMP or Lightroom round trip | ✗ No | — |
| OpenEXR, Radiance HDR, or PFM export | ✗ No | — |
| HDR or extended-dynamic-range preview and export | ✗ No | No gain-map or scene-linear output |

## Platform and operation

| Capability | Available | Notes |
| --- | :---: | --- |
| Multiple Immich users | ✓ Yes | Local edits and jobs are isolated by user |
| Docker on Linux amd64 and arm64 | ✓ Yes | Multi-architecture image |
| Vulkan GPU rendering on Linux | ✓ Yes | Requires GPU passthrough |
| Metal GPU rendering on macOS | ✓ Yes | Native process only |
| CPU rendering fallback | ✓ Yes | Available on every supported deployment |
| Local AI inference | ✓ Yes | Models run on the server; no cloud service |
| Full mobile editor | ✗ No | The editor requires a desktop-width viewport |
