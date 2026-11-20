# <img src="web/static/favicon.svg" width="24" height="24" alt=""> immich-edit

A self-hosted, non-destructive RAW editor for an existing
[Immich](https://immich.app/) library. It browses assets through Immich, decodes and exports on the
server, draws live previews in the browser when WebGPU is available, and stores edits locally.
Originals remain unchanged, and go to the Immich trash only when you ask.

> [!WARNING]
> immich-edit is under active `0.x` development. Back up `DATA_DIR` and read the upgrade notes
> before upgrading.

![immich-edit on desktop, tablet and phone](docs/image/banner-v0_8.png)


<!-- SCREENSHOT: captured from the maintainer's library on the dev server (album "Safari 2025", each photo developed with Auto and lens corrections on, account shown as Hanne Kristiansen at photos.example.net): desktop album grid 1440x900 sorted by rating, tablet editor 1180x820, phone editor 412x839, composed into a 1600x840 banner at scale 2. -->

## Documentation

- [Getting started](https://haavardnk.github.io/immich-edit/getting-started/)
- [Use the editor](https://haavardnk.github.io/immich-edit/use/): culling, editing and exporting
- [Run the server](https://haavardnk.github.io/immich-edit/operate/): HTTPS, GPU passthrough,
  OAuth, configuration and backups
- [Troubleshooting](https://haavardnk.github.io/immich-edit/troubleshooting/)
- [Feature matrix](https://haavardnk.github.io/immich-edit/features/)

## Project

Ask usage and setup questions in
[GitHub Discussions](https://github.com/haavardnk/immich-edit/discussions). Use
[GitHub Issues](https://github.com/haavardnk/immich-edit/issues) for reproducible bugs and feature
requests. Read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting a change. Report vulnerabilities
through [GitHub Security Advisories](https://github.com/haavardnk/immich-edit/security/advisories/new).

Community support is best effort, with no response-time guarantee.

Licensed under [AGPL-3.0-only](LICENSE).
See the [attributions](https://haavardnk.github.io/immich-edit/attributions/) for projects and
bundled camera profiles used by immich-edit.
