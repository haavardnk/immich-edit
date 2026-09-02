---
layout: default
title: Attributions
nav_order: 17
permalink: /attributions/
---

# Attributions

immich-edit builds on these projects:

- [Immich](https://immich.app/) provides the photo library and API. The interface is built with its
  [`@immich/ui`](https://github.com/immich-app/static-pages/tree/main/packages/ui) component library.
- [Bits UI](https://bits-ui.com/) provides accessible Svelte interaction primitives under the MIT
  license.
- [Material Design Icons](https://pictogrammers.com/library/mdi/) provides interface icons through
  `@mdi/js` under the Apache-2.0 license.
- [uuid](https://github.com/uuidjs/uuid) generates browser-side identifiers under the MIT license.
- [RapidRAW](https://github.com/CyberTimon/RapidRAW) inspired parts of the editing pipeline.
- [rawler](https://github.com/dnglab/dnglab) parses RAW files.
- [wgpu](https://wgpu.rs/) runs GPU rendering on Vulkan and Metal.
- [ONNX Runtime](https://onnxruntime.ai/) runs local mask models.
- [lensfun](https://lensfun.github.io/) provides lens correction data.
- [justified-layout](https://github.com/flickr/justified-layout) arranges mixed-aspect photo rows.

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

## Bundled fonts

The interface uses [Google Sans](https://github.com/googlefonts/googlesans) v14.000 and
[Google Sans Code](https://github.com/googlefonts/googlesans-code) v7.001. Both fonts are
distributed under the SIL Open Font License 1.1. Their copyright, license, and trademark notices
are included under `web/static/licenses/`.

The `justified-layout` license notice is also included under `web/static/licenses/`.

## Project license

immich-edit is distributed under
[AGPL-3.0-only](https://github.com/haavardnk/immich-edit/blob/main/LICENSE).
