---
layout: default
title: Export
parent: Use the editor
nav_order: 3
permalink: /export/
---

# Export

An export renders the edited photo from the original at full resolution, on the server. It either
downloads to your computer or uploads to Immich as a new photo next to the original.

## Export one photo

1. Open **Export** in the editor, or press `Ctrl+Shift+E`.
1. Choose **Download** or **To Immich**.
1. Pick a **Format** and its options.
1. Press the button at the bottom. It names the format, such as **Export JPEG** or
   **Upload JPEG to Immich**.

The defaults, JPEG at quality 90 in sRGB with EXIF metadata, suit sharing and viewing in Immich.

## Format options

| Option | Shown for | Notes |
| --- | --- | --- |
| **Format** | Always | **JPEG**, **PNG**, **WebP**, **AVIF**, **HEIC**, **TIFF** or **JPEG XL** |
| **Color space** | Always | **sRGB** for the web and most screens. **Display P3** keeps more saturated color for wide-gamut screens |
| **Quality** | JPEG, AVIF, HEIC, lossy WebP | 1 to 100 |
| **Bit depth** | PNG, TIFF, JPEG XL | **16-bit** keeps smooth gradients through later editing elsewhere |
| **Compression** | PNG, TIFF | Trades file size against save time; all choices are lossless |
| **Lossless** | WebP | Forced on while **Include EXIF metadata** is checked |
| **Include EXIF metadata** | Always | Copies camera, lens, date and location. Embedded previews are left out |

Output always has the edited photo's full size. Exports cannot be resized or watermarked;
[features](features.md#export-color-and-interoperability) lists what is and is not supported.
[Compatibility](compatibility.md#image-input-and-export) lists the bit depths each format
supports.

## Name the file

**Filename** takes a template that names every export, whether it downloads, goes into a ZIP or
uploads to Immich. It starts as `{name}_edit`. Under the field you see the name the current photo
will get.

| Token | Becomes |
| --- | --- |
| `{name}` | The original's name without its extension |
| `{date}` | The capture date as `YYYY-MM-DD`, from the camera's local time. A photo without one uses the file's date |
| `{seq}` | The photo's place in the export, from 1. It gets leading zeros to match the count, so 120 photos run `001` to `120`. A single export is `1` |

Text around the tokens is kept, so `{date}_{name}` gives `2024-05-01_IMG_0001.jpg` and
`trip_{seq}` gives `trip_007.jpg`. The extension comes from the format. Characters that are
not allowed in file names, such as `/`, `:` or `*`, are refused in the template and replaced by
`_` when they come from the original's name. If a name is already taken, next to the original
in Immich or earlier in the same ZIP, it gets `_2`, `_3` and so on. An unknown token or an
unmatched brace shows an error under the field and blocks the export until you fix it.

## Upload to Immich

**To Immich** adds these options:

- **Albums** and **Tags** add the new photo to existing Immich albums and tags.
- **Mark as favorite** favorites the new photo.
- **Stack with original** stacks the new photo with the original in Immich. **Edit primary** shows
  the edit on top of the stack, and **Original primary** keeps the original on top.

Immich refuses a file it already has. Uploading an identical export again reports
`Not uploaded: identical asset already exists in Immich (matched by content hash)`.
Change the edit or a format option to upload another one. A failed upload shows **Retry**.

The edit stays in immich-edit after the upload. Change it later and export again for an updated
file; the earlier upload stays in Immich until you delete it there.

## Export many photos

1. [Select the photos](cull.md#choose-between-similar-shots) in the grid.
1. Press **Edit and export selected**.
1. On the **Export** tab, choose **Download ZIP** or **To Immich** and set the options as for a
   single photo.
1. Press **Download** *count* **as** *format* **ZIP** or **Export** *count* **to Immich**.

The export runs on the server as a job, so you can keep working or close the tab. **Jobs**, in the
top bar, shows progress with a count of running jobs. Expand a job to see each photo and the reason
any failed. The cancel button stops a running job. A finished ZIP job has a **Download ZIP** button.
**Clear finished** tidies the list.

The **Edit** tab of the same dialog works on the whole selection too:

- **Copy** takes the edits from a single selected photo and opens **Copy settings**.
- **Paste** applies the copied edits to every selected photo.
- **Presets** applies a preset to every selected photo, at the **Amount** you choose.
- **Reset** returns every selected photo to its original state.

These also run as jobs.

## Large files and slow servers

A full-resolution export decodes, renders and encodes the original in one request. On a server
without a GPU a large RAW file can take minutes. If exports stop with `408 Request Timeout`, see
[an export returns 408](troubleshooting.md#an-export-returns-408-request-timeout).
