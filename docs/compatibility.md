---
layout: default
title: Compatibility
nav_order: 13
permalink: /compatibility/
---

# Compatibility

## RAW input

RAW coverage follows rawler's decoder set. A format listed as supported by rawler may still need a
camera-specific fix. The rows marked tested have committed end-to-end fixtures.

| Vendor | Formats | Status |
| --- | --- | --- |
| Sony | `.arw`, `.sr2`, `.srf` | `.arw` tested |
| Canon | `.cr2`, `.cr3`, `.crw` | `.cr2` and `.cr3` tested |
| Nikon | `.nef`, `.nrw` | `.nef` tested |
| Fujifilm | `.raf` | Bayer and X-Trans paths tested |
| Panasonic | `.rw2` | Tested |
| Olympus / OM System | `.orf` | Tested |
| Pentax | `.pef`, `.dng` | Tested |
| Leica | `.rwl`, `.dng` | Tested |
| Hasselblad | `.3fr`, `.fff` | Decoder support |
| Phase One | `.iiq` | Decoder support |
| Ricoh | `.dng` | Tested |
| Adobe DNG | Linear and Bayer `.dng` | Tested |
| Sigma | `.x3f` | Not supported |

Pixel-shift and multi-frame files decode only their first sub-frame. Bayer and X-Trans sensors are
supported; an unusual color filter array outside those is rejected. Fujifilm X-Trans uses a
single-pass color-difference demosaic and can be softer than a multi-pass converter on fine detail.

Recent bodies come with the decoder library and arrive as it is updated. The current version adds
the Sony A7V, A7R VI, RX1R III and RX10 V, Nikon ZR, Fujifilm X-T30 III, Canon R50 V, C50 and
PowerShot V1, OM System OM-5 II, Olympus E-P7, Leica SL3-P, and several Panasonic S and G models.
It also corrects the Olympus E-M1X color filter array, Panasonic black levels, Nikon and Sony A100
white balance, and Canon CR3 preview extraction, so files from those cameras can render slightly
differently than before.

## Image input and export

| Format | Input | Export | Export depth |
| --- | :---: | :---: | --- |
| DNG | ✓ | ✗ | — |
| JPEG | ✓ | ✓ | 8-bit |
| PNG | ✓ | ✓ | 8-bit or 16-bit |
| TIFF | ✓ | ✓ | 8-bit or 16-bit |
| WebP | ✓ | ✓ | 8-bit |
| HEIC / HEIF | ✓ | ✓ | 8-bit |
| AVIF | ✓ | ✓ | 8-bit |
| JPEG XL | ✓ | ✓ | 8-bit or 16-bit |
| GIF | ✓ | ✗ | — |
| BMP | ✓ | ✗ | — |
| PSD / PSB | ✗ | ✗ | — |
| OpenEXR | ✗ | ✗ | — |
| Radiance HDR / PFM | ✗ | ✗ | — |

PNG and TIFF input accepts 8-bit or 16-bit RGB, RGBA, and grayscale. Floating-point and CMYK TIFF,
plus some tiled TIFF layouts, are not supported. HEIC and AVIF need separate libheif decoder and
encoder plugins; check **Settings** > **Diagnostics**. The official image includes all four
capabilities.

Animated and multipage containers are treated as still images. GIF, TIFF, HEIF, and similar files
decode one image or frame.

Exports can preserve safe EXIF metadata. Output color is sRGB or Display P3; Adobe RGB, ProPhoto
RGB, Rec.2020, custom ICC profiles, linear output, and HDR gain maps are not supported.

## Platforms

| Host | Image rendering | AI masks |
| --- | --- | --- |
| Linux amd64 with Vulkan GPU | GPU, CPU fallback | WebGPU, CPU fallback |
| Linux amd64 without GPU | CPU | CPU; x86-64-v3 CPU required by ONNX Runtime |
| Linux arm64 with Vulkan GPU | GPU, CPU fallback | CPU only |
| Linux arm64 without GPU | CPU | CPU |
| macOS native | Metal, CPU fallback | WebGPU, CPU fallback |
| macOS Docker | CPU | CPU |

The Docker image supports Linux `amd64` and `arm64`. Metal cannot pass through the Linux virtual
machine used by Docker on macOS.

Hosts with no graphics hardware still get a Vulkan device from Mesa's software rasterizer, llvmpipe
or lavapipe. The backend uses it and Diagnostics marks it as software. It is not hardware
acceleration, but it does render preview-sized frames faster than the built-in CPU renderer.

## Browser and viewport

The editor requires a desktop-width viewport and has no full mobile layout. RAW decoding and the
sensor stage run on the server. With WebGPU (current Chromium, Edge and Safari) the browser draws
the fit-view preview itself; without it, or when **Settings** > **Preview rendering** is set to
**Server**, the server renders every preview, so WebGPU is optional.
