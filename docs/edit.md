---
layout: default
title: Edit a photo
parent: Use the editor
nav_order: 2
permalink: /edit/
---

# Edit a photo

Every change saves by itself, and the original in Immich never changes. Keys are listed on
[keyboard shortcuts](shortcuts.md).

## Find your way around

The tool rail switches between **Develop**, **Masks**, **Retouch**, **Crop** and **Export** (see
[export](export.md)). `←` and `→` move between photos and `G` goes back to the grid. `Ctrl+K`
opens the command palette, which finds settings, presets and actions.

On a phone or tablet the same tools sit in a bar at the bottom with their controls in a tray, or
in a side panel in landscape. **Share** in the top bar opens export, and **More editor actions**
holds **History**, **Snapshots** and **Versions and copies**.

## Develop a photo

Work through **Develop** from the top:

1. **Camera Profile**: **Auto match** suits most RAW files.
1. **Basic**: **Auto** (`Ctrl+U`) sets white balance and tone as a starting point. Then adjust
   **White Balance**, **Tone** (**Exposure** first) and **Presence**.
1. **Curves**, **HSL**, **Color Grading** and **LUT** shape color and contrast.
1. **Black & White**; see [black and white](#black-and-white).
1. **Detail**: sharpening, noise reduction and **AI Denoise**. Judge them at 100% zoom (`Z`).
1. **Lens Corrections** fixes distortion and vignetting from the lens profile.
1. **Lens Blur** blurs the background from a depth map (**Generate depth**), and **Effects** adds
   a vignette and grain.

AI Denoise, Lens Blur and AI masks need models that an administrator installs; see
[models](administration.md#models).

Double-click a slider's label to reset it, or click its value to type a number. Right-click a
slider, or hold it on a touch screen, and choose **Pin to Quick** to keep it in the **Quick**
panel. **Search settings** at the top of **Develop** finds any slider. **Adjust on photo** (`T`)
in **HSL**, **Curves**, **Black & White** and **Color Grading** lets you drag on the photo to
change the color or tone under the pointer.

### Black and white

**Convert to Black & White** turns the photo monochrome. The color sliders make each color lighter
or darker in the gray version, and the **Shadows** and **Highlights** wheels tint it.

## Check your work

Hold **Hold for original**, `\` or `<` to see the unedited photo. `Y` cycles a split and a
side-by-side compare against the original, a snapshot or another version. **More editor actions**
has the clipping overlay (`J`), **Grey background**, the scope overlay (`S`) and **Soft proof**;
the scopes sit at the top of **Develop**.

## Undo and history

`Ctrl+Z` undoes and `Ctrl+Shift+Z` redoes. **Edit history** lists every saved state; click one to
jump back, and your next edit drops the newer ones. **New snapshot** (`Ctrl+Shift+S`) saves the
current edits under a name. **Reset** (`Ctrl+Shift+R`) returns every Develop panel to its default.

## Adjust part of the photo

**Masks** applies adjustments to one area:

1. Press **New** and pick a shape: **Linear gradient**, **Radial gradient**, **Brush**,
   **Polygon**, **Luminance range**, **Color range**, or an AI selection such as **Subject**,
   **Sky** or **Click to select**.
1. Set the mask's adjustments below the list.

**Refine** combines shapes with **Add**, **Subtract** or **Intersect**, and **Invert** flips the
mask. `O` shows the selected mask as an overlay.

## Remove spots and distractions

Open **Retouch**, pick **Heal** or **Clone**, hold `Alt` and click a clean area as the source,
then paint over the spot. Drag a stroke's circle to move its source.

## Crop and straighten

**Crop** has aspect ratio chips (and **Ratio** for any other shape), **Angle** and **Straighten**
(`A`) to level the horizon, rotate and flip buttons, and **Vertical** and **Horizontal** for
converging lines. **Guided upright** sets them from lines you draw along edges that should be
straight. **Crop guide** (`O`) cycles the overlay.

## Reuse edits

**Copy edits** (`Ctrl+C`) lets you pick the sections to copy, and **Paste edits** (`Ctrl+V`)
applies them to another photo, or to every photo selected in the grid. **Presets** (`Shift+P`)
shows a strip of saved looks: point at one to preview, click to apply, and set **Amount** to
apply it more or less strongly. **Save current as preset** saves the current look.

## Try variations

**Versions** in **More editor actions** lists the original and its virtual copies. **Create
virtual copy** (`Ctrl+'`) starts a new version with its own edits; hold another version to
preview it. See also [virtual copies](cull.md#virtual-copies).
