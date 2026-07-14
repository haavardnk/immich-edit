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

`RUSTSEC-2026-0194` and `RUSTSEC-2026-0195` are temporarily excluded from the automated Rust audit. They affect XML attribute and namespace parsing in `quick-xml` through XMP code in `little_exif`. immich-edit uses EXIF read/write paths and does not parse XMP through those paths. The exceptions should be removed when `little_exif` supports `quick-xml` 0.41 or newer, or before adding XMP parsing.
