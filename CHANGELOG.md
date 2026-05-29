# Changelog

All notable changes to immich-edit are listed here. Format loosely follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Pre-1.0 means anything can break between `0.x` bumps (e.g. `0.1.0` to `0.2.0`), and sidecar/db schemas may require a cache wipe.

## Unreleased

## 0.1.0-beta.1 - 2026-05-29

First public beta. Published as `haavardnk/immich-edit` on Docker Hub.

### What works

- RAW decoding via rawler for the formats it supports (.arw, .cr2, .cr3, .nef, .dng, and others).
- Standard image formats for input and output: JPEG, PNG, TIFF, WebP, HEIC, AVIF, JPEG XL, GIF, BMP.
- GPU rendering through wgpu (Vulkan on Linux, Metal on macOS), with a CPU fallback.
- Basic, tone, color grading, HSL, curves, detail, effects, lens corrections (lensfun), and display panels.
- Local masks: radial, linear, brush. Crop, rotate, flip.
- Edit history per asset with restore.
- Export to disk or back to Immich as a new asset.
- Single-user token auth with reverse-proxy friendly behaviour.

### Known limits

- Single-user only. No accounts.
- No HDR output, DNG export, PSD compatibility, LUTs, AI features, or presets.
- No mobile editor layout.
- Sidecar/DB schemas can change between betas. Back up `cache/immich-edit.db` before upgrading.
