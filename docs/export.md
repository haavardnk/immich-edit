---
layout: default
title: Export
parent: Use the editor
nav_order: 3
permalink: /export/
---

# Export

An export renders the edited photo from the original at full resolution on the server. It
downloads to your device or uploads to Immich as a new photo next to the original.

## Export one photo

1. Open **Export** in the editor (`Shift+E`), or tap **Share** on a phone or tablet.
1. Choose **Download** or **To Immich**, then a **Format** and its options.
1. Press the button at the bottom, such as **Export JPEG** or **Upload JPEG to Immich**.

The default is JPEG at quality 90 in sRGB with EXIF metadata.

### Save, share or copy the file

A download waits in a card until you press **Save**. The card also offers **Copy image**,
**Share** and **Upload to Immich**. **Save automatically** downloads every export as it renders.

## Format options

**Format** is **JPEG**, **PNG**, **WebP**, **AVIF**, **HEIC**, **TIFF** or **JPEG XL**, with
**Quality**, **Bit depth** or **Compression** where they apply. **Color space** is **sRGB** or
**Display P3**. **Metadata** copies **All**, **All but location** or **None** of the EXIF.
[Features](features.md#export-color-and-interoperability) and
[compatibility](compatibility.md#image-input-and-export) list what each format supports.

## Resize

**Resize** keeps **Full size** or scales to **Dimensions**, **Megapixels** or a **Percentage**.
**Don't enlarge** keeps a smaller photo at its own size.

## Output sharpening

**Sharpen** adds sharpening for the output after the resize: **Screen**, **Matte paper** or
**Glossy paper**, at an **Amount**, with the **Print PPI** for paper. It never changes the saved
edit.

**Watermark** stamps a PNG from the shared library onto the export, with **Size**, **Opacity**,
**Inset** and **Position**. Only an administrator can add watermarks.

## Name the file

**Filename** is a template, `{name}_edit` by default. The tokens are `{name}`, `{date}`, `{seq}`
and `{preset}`; the chips under the field insert them. A taken name gets `_2`, `_3` and so on.

## Upload to Immich

**To Immich** adds **Albums**, **Tags**, **Mark as favorite**, **Stack with original** and
**Replace original**. The upload lands next to the original in the timeline and gets the date,
place, description and rating Immich shows for the original.

### Export a photo again

When a photo was uploaded before, a card offers **Keep both** (the default), or replacing or
skipping the earlier export.

### Replace the original

**Replace original** keeps only the edit: the upload takes over the original's albums, shared
links and stack, and the original goes to the Immich trash. **Undo** brings it back.

{: .warning }
Immich deletes a trashed original for good when its trash period ends, and a RAW original then
survives only as the exported file. To edit the photo again, restore the original from the Immich
trash first.

## Export many photos

Select photos in the grid ([how](cull.md#choose-between-similar-shots)) and press **Export
selected photos** (`Shift+E`). Set the options as for one photo and start the export; it runs as
a job on the server, shown under **Jobs**, so you can keep working. A ZIP downloads when it is
ready, and **Retry** runs failed photos again. **Edits** in the selection bar applies a preset,
pastes or resets edits on every selected photo.

## Export presets

**Export presets** saves the current export settings under a name (**Save current as…**) and
brings them back with **Apply**. Presets belong to your account.

### Export with several presets

Tick several presets and press **Export with** *count* **presets** to export each photo once per
preset.

## Large files and slow servers

A full-resolution export of a large RAW file can take minutes on a server without a GPU. See
[an export returns 408](troubleshooting.md#an-export-returns-408-request-timeout) and
[a photo is too large](troubleshooting.md#a-photo-is-too-large-for-the-servers-memory).
