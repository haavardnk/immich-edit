# Security policy

## Supported versions

Only the latest release receives security updates. Fixes ship as a new release rather than as a
patch to an older tag, so upgrade to the newest version listed on the
[releases page](https://github.com/haavardnk/immich-edit/releases).

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

`cargo audit` runs in CI and a few advisories are excluded from it. The authoritative list is the
`ignore:` line in [`.github/workflows/audit.yml`](.github/workflows/audit.yml), where each entry
carries the reason it is there. An exception is only acceptable while the advisory cannot reach a
code path immich-edit uses, and is removed as soon as an upgrade clears it.
