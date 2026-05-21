# immich-edit

Self-hosted web RAW editor for [Immich](https://immich.app/). Browses Immich albums, renders RAW originals to JPEG previews with GPU acceleration, and supports basic non-destructive edits.

Status: pre-alpha. Nothing here works yet.

## Stack

- Rust backend (`axum`, `rawler`, `wgpu`, `turbojpeg`)
- SvelteKit + TypeScript + Tailwind v4 + daisyUI v5 frontend
- Single Docker image, CPU default, optional GPU

## License

AGPL-3.0-only
