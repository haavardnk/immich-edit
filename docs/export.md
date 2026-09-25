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

The options are grouped into **File**, **Size**, **Watermark** and **Name**, plus **Immich** when
you upload. The sections below follow the same order.

The defaults, JPEG at quality 90 in sRGB with EXIF metadata, suit sharing and viewing in Immich.

## Format options

| Option | Shown for | Notes |
| --- | --- | --- |
| **Format** | Always | **JPEG**, **PNG**, **WebP**, **AVIF**, **HEIC**, **TIFF** or **JPEG XL** |
| **Quality** | JPEG, AVIF, HEIC, lossy WebP | 1 to 100 |
| **Bit depth** | PNG, TIFF, JPEG XL | **16-bit** keeps smooth gradients through later editing elsewhere |
| **Compression** | PNG, TIFF | Trades file size against save time; all choices are lossless |
| **Lossless** | WebP | Forced on while **Include EXIF metadata** is checked |
| **Color space** | Always | **sRGB** for the web and most screens. **Display P3** keeps more saturated color for wide-gamut screens |
| **Include EXIF metadata** | Always | Copies camera, lens, date and location. Embedded previews are left out. In a JPEG, EXIF over 64 KB loses maker notes and user comments first, then everything but capture, copyright and location; other formats keep it all |

[Features](features.md#export-color-and-interoperability) lists what is and is not supported.
[Compatibility](compatibility.md#image-input-and-export) lists the bit depths each format
supports.

## Resize

**Resize** starts at **Full size**, the edited photo's own resolution after the crop. The other
choices scale the export:

| Choice | You enter | Result |
| --- | --- | --- |
| **Dimensions** | Width × height in pixels | The photo fits inside that size and keeps its shape |
| **Megapixels** | Megapixels | The photo keeps its shape and has about this many pixels in total |
| **Percentage** | 1 to 400 | Both sides scale by this much; above 100 enlarges |

In the editor, width and height are linked to the crop's shape. **Dimensions** starts at the
crop's own size, and typing one side fills in the other, so the export comes out at exactly the
size you see. In a bulk export the photos have different shapes, so the two numbers are a box each
photo fits inside. Leave one side empty to limit only the other.

**Don't enlarge** is on by default for **Dimensions** and **Megapixels**, so a photo that is
already smaller than the size you ask for keeps its own size. Turn it off to scale small crops up;
the enlarged photo is resampled once, at the end, after every edit. In the editor, the **Size**
line under the resize options shows the size the export will have. The largest export is 65,535
pixels on the long edge.

## Output sharpening

Resizing and printing both soften a photo. Output sharpening, the **Sharpen** option, puts back
what the output loses. It runs once, on the finished pixels, after the resize, so it matches the
export's final size. It never touches the editor preview or the saved edit.

| Choice | Use it for |
| --- | --- |
| **None** | No extra sharpening (the default) |
| **Screen** | Web, social media and phone or monitor viewing |
| **Matte paper** | Matte and fine-art paper, which spreads ink the most |
| **Glossy paper** | Glossy, lustre and other coated paper |

**Amount** is **Low**, **Standard** or **High**. For the two paper choices, enter the
**Print PPI** (print resolution) your print will have, from 72 to 1200 (300 by default). A higher
resolution sharpens with a wider radius, because the same ink spread on paper covers more of the
smaller printed pixels. Sharpening works on brightness only, so it does not add color fringes,
and it never pushes a pixel past its neighbors, so edges do not get bright or dark halos. The
Detail panel's Sharpen is still the place to sharpen the photo itself; output sharpening is for
the output.

## Watermark

**Watermark** stamps a PNG from the shared watermark library onto the export, such as a logo, a
signature or a copyright line saved with a transparent background. It goes on last, after the
resize and output sharpening, so it is never resampled with the photo or sharpened. It never
touches the editor preview or the saved edit.

| Control | What it does |
| --- | --- |
| **Image** | **None** (the default) or a PNG from the library |
| **Size** | The watermark's longer side as a share of the photo's shorter side, 1 to 100% (20% by default) |
| **Opacity** | 1 to 100% (80% by default), on top of the PNG's own transparency |
| **Inset** | Distance from the nearest edges as a share of the photo's shorter side, 0 to 50% (3% by default) |
| **Position** | One of nine spots: a corner, the middle of an edge, or the center |

Size and inset follow the photo's shorter side, so the watermark keeps its proportion in portrait
and landscape photos and at every export size. Watermark PNGs are treated as sRGB, and a
Display P3 export converts them so their colors still match.

The library is shared by every user. Only an administrator can add or remove watermarks, with
the **Import PNG watermark** and **Delete watermark** buttons beside the picker. A PNG may be up to
16 MB and 4096 pixels on its longest side. Deleting a watermark removes it from the picker;
exports already queued with it still finish.

## Name the file

**Filename** takes a template that names every export, whether it downloads, goes into a ZIP or
uploads to Immich. It starts as `{name}_edit`. Under the field you see the name the current photo
will get, and the list of tokens while the field is focused.

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
