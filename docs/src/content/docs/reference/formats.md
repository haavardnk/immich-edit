---
title: Supported formats
description: Supported RAW cameras, image input/export formats, color spaces, and platform limits.
---

## RAW input

RAW decoding uses `rawler`. Rows marked **Tested** have committed end-to-end fixtures.

| Vendor | Formats | Status |
| --- | --- | --- |
| Sony | `.arw`, `.sr2`, `.srf` | `.arw` tested |
| Canon | `.cr2`, `.cr3`, `.crw` | `.cr2`, `.cr3` tested |
| Nikon | `.nef`, `.nrw` | `.nef` tested |
| Fujifilm | `.raf` | Bayer and X-Trans tested |
| Panasonic | `.rw2` | Tested |
| Olympus / OM System | `.orf` | Tested |
| Pentax | `.pef`, `.dng` | Tested |
| Leica | `.rwl`, `.dng` | Tested |
| Hasselblad | `.3fr`, `.fff` | Decoder support |
| Phase One | `.iiq` | Decoder support |
| Ricoh | `.dng` | Tested |
| Apple, Google, LG, other phones | `.dng` | iPhone 8 & Nexus 5X tested; ProRAW decodes |
| DJI | `.dng` | Mini 2 tested |
| Adobe DNG | Linear and Bayer `.dng` | Tested |
| Sigma | `.x3f` | Not supported |

**Notes:**
- Phone and drone DNG lens shading, vignette, and distortion opcodes are always applied; fisheye
  warps, map tables, and iPhone ProRAW profile gain table maps are skipped.
- Pixel-shift and multi-frame RAWs decode only their first sub-frame.
- Only 2×2 Bayer and 6×6 X-Trans CFAs are supported.

## Image input and export

| Format | Input | Export | Export bit depth |
| --- | :---: | :---: | --- |
| DNG | ✓ | ✗ | — |
| JPEG | ✓ | ✓ | 8-bit |
| PNG | ✓ | ✓ | 8-bit or 16-bit |
| TIFF | ✓ | ✓ | 8-bit or 16-bit |
| WebP | ✓ | ✓ | 8-bit (lossy or lossless) |
| HEIC / HEIF | ✓ | ✓ | 8-bit |
| AVIF | ✓ | ✓ | 8-bit |
| JPEG XL | ✓ | ✓ | 8-bit or 16-bit |
| GIF, BMP | ✓ | ✗ | — |
| PSD, PSB, OpenEXR, HDR, PFM | ✗ | ✗ | — |

- PNG and TIFF input accept integer 8/16-bit RGB, RGBA, and grayscale (no float, CMYK, or
  unsupported tiled TIFF).
- Animated or multipage containers decode a single still frame.
- Exports support **sRGB** and **Display P3** (with embedded ICC profiles) and up to 65,535 px on
  the long edge.

## Platforms and browsers

| Host | Image rendering | AI models |
| --- | --- | --- |
| Linux `amd64` + Vulkan GPU | GPU, CPU fallback | WebGPU, CPU fallback |
| Linux `amd64` (CPU only) | Software rasterizer (`llvmpipe`), CPU fallback | CPU (`x86-64-v3` required) |
| Linux `arm64` + Vulkan GPU | GPU, CPU fallback | CPU only |
| Linux `arm64` (CPU only) | Software rasterizer (`llvmpipe`), CPU fallback | CPU only |
| macOS native | Metal, CPU fallback | WebGPU, CPU fallback |
| macOS Docker | Software rasterizer (`llvmpipe`), CPU fallback | CPU only |

Desktop, tablet, and phone viewports are all supported. Browsers with WebGPU (current Chromium,
Edge, Safari) draw local previews over HTTPS or `localhost`; other browsers and plain HTTP fall back
to server previews seamlessly.
