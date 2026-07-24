# Bundled DNG Camera Profiles (DCP)

The `.dcp` files in this directory are third-party DNG Camera Profiles bundled
with immich-edit for camera-model auto-matching.

## Source

RawTherapee — https://github.com/Beep6581/RawTherapee
Path: `rtdata/dcpprofiles/`

## License

These profiles are distributed as part of the RawTherapee project under the
**GNU General Public License v3.0 or later** (GPL-3.0-or-later). immich-edit is
licensed under AGPL-3.0, which is compatible with GPLv3, so redistribution here
is permitted with attribution.

Some individual profiles carry a `ProfileCopyright` tag of **CC0-1.0** or
**public domain**; those are additionally free of attribution requirements. The
per-profile copyright is preserved in `manifest.json`.

- GPL-3.0: https://www.gnu.org/licenses/gpl-3.0.html
- CC0-1.0: https://creativecommons.org/publicdomain/zero/1.0/

## Notes

- RawTherapee stores these profiles as standard DNG DCP TIFF containers with a
  custom version magic (`RC` instead of `0x002A`); immich-edit reads both.
- See `manifest.json` for the full per-profile list (file, camera model,
  copyright).
