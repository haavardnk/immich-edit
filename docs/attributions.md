---
layout: default
title: Attributions
nav_order: 17
permalink: /attributions/
---

# Attributions

immich-edit builds on these projects:

- [Immich](https://immich.app/) provides the photo library and API.
- [RapidRAW](https://github.com/CyberTimon/RapidRAW) inspired parts of the editing pipeline.
- [rawler](https://github.com/dnglab/dnglab) parses RAW files.
- [wgpu](https://wgpu.rs/) runs GPU rendering on Vulkan and Metal.
- [ONNX Runtime](https://onnxruntime.ai/) runs local mask models.
- [lensfun](https://lensfun.github.io/) provides lens correction data.

## Bundled camera profiles

Bundled DNG camera profiles come from
[RawTherapee](https://github.com/Beep6581/RawTherapee), revision
`039b9b89d43315be6b42e8fbb33b8cfb39edd4bf`. They are distributed under
GPL-3.0-or-later. Some profiles are CC0 or public domain; each profile's copyright remains in the
bundled manifest.

See the
[full profile notice](https://github.com/haavardnk/immich-edit/blob/main/crates/backend/assets/dcp/NOTICE.md)
and
[profile manifest](https://github.com/haavardnk/immich-edit/blob/main/crates/backend/assets/dcp/manifest.json).

## Project license

immich-edit is distributed under
[AGPL-3.0-only](https://github.com/haavardnk/immich-edit/blob/main/LICENSE).
