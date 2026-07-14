# Security policy

## Supported versions

Only the latest released version receives security updates during the `0.x` release line.

## Reporting a vulnerability

Report security issues privately through GitHub Security Advisories:

[Open a private security advisory](https://github.com/haavardnk/immich-edit/security/advisories/new).

Do not open public issues for security problems. I try to respond within 7 days.

Please include:

- A description of the issue and its impact
- Steps to reproduce or a proof of concept
- Affected versions and your environment

Please give me time to fix the issue before publishing details. I will publish a fix and advisory once a patched release is available.

## Dependency audit exceptions

`RUSTSEC-2026-0194` is temporarily excluded from the automated Rust audit. It affects XML attribute parsing in `quick-xml` through an unused XMP cleanup helper in `little_exif`. immich-edit does not parse XMP through that helper. The exception should be removed when `little_exif` supports `quick-xml` 0.41 or newer, or before adding XMP parsing.
