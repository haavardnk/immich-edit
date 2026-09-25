---
layout: default
title: Edit a photo
parent: Use the editor
nav_order: 2
permalink: /edit/
---

# Edit a photo

Every change saves by itself; the status in the bottom bar reads **Saving…** and then **Saved**.
Edits are stored in immich-edit, so the original in Immich never changes, and you can always go
back.

## Find your way around

The tool rail on the right switches between five tools:

| Tool | Key | Use it for |
| --- | --- | --- |
| **Develop** | `D` | Color, tone, detail, lens, presets and versions |
| **Masks** | `M` | Adjustments that apply to part of the photo |
| **Retouch** | `Q` | Removing dust, spots and distractions |
| **Geometry** | `R` | Crop, straighten, rotate and perspective |
| **Export** | `Ctrl+Shift+E` | Downloading or uploading the result; see [export](export.md) |

`←` and `→` move to the previous or next photo, and `G` goes back to the grid. `Tab` hides the
side panels, `Shift+Tab` hides every panel, and `Shift+F` goes full screen.

## Develop a photo

**Develop** is a stack of panels. Working from the top down suits most photos:

1. **Camera Profile** decides how the camera's colors are read. **Auto match** picks the profile for
   your camera and is the right start for most RAW files. **Flat** gives a low-contrast base for
   heavy grading.
1. **Basic** does most of the work. **Auto**, at the top of **Develop** or `Ctrl+U`, sets tone as a
   starting point. Then:
   - **White Balance**: **Pick white balance** and click something that should be neutral gray, or
     use **Auto white balance**, then fine-tune **Temperature** and **Tint**.
   - **Tone**: set **Exposure** first, recover skies with **Highlights**, open up shadows with
     **Shadows**, and set the end points with **Whites** and **Blacks**.
   - **Presence**: **Texture** and **Clarity** add bite, **Dehaze** cuts haze, and **Vibrance**
     boosts muted colors more gently than **Saturation**.
1. **Curves**, **HSL** and **Color Grading** shape color and contrast further. **LUT** applies a
   creative look from a `.cube` file, with an **Amount**.
1. **Black & White** turns the photo monochrome; see [black and white](#black-and-white).
1. **Detail** holds sharpening and noise reduction. Judge both at 100% zoom (`Z`).
1. **Lens Corrections** fixes distortion and vignetting from the lens profile of a RAW file.
   **Remove Chromatic Aberration** is off until you turn it on.
1. **Effects** adds a vignette and grain.

**Presets** and **Versions** sit in the same stack; see [reuse edits](#reuse-edits) and
[try variations](#try-variations).

### Sliders and panels

- Double-click a slider's label to reset it.
- Click a slider's value to type an exact number.
- With a slider focused, `Shift+←` and `Shift+→` move it ten steps.
- Hold `Alt` while dragging a slider that drives a mask or a radius to see what it covers.
- A panel with changes shows a dot next to its name. Click the dot to reset the whole panel.
- Press and hold a changed section's name, such as **Tone**, to see the photo without that section.
- **Show modified only**, in the row at the top of **Develop**, hides every untouched panel.

### Black and white

**Convert to Black & White** in the **Black & White** panel replaces the colors with their
brightness. It runs after **Basic** and **HSL**, so their color changes still decide what each
color turns into, and before **Color Grading** and **LUT**, which can tint the result.

| Control | What it does |
| --- | --- |
| **Red**, **Yellow**, **Green**, **Aqua**, **Blue**, **Magenta** | Makes that color lighter or darker in the gray version, up to about 1.5 stops each way. Neutral grays never move, and faint colors move less than strong ones |
| **Shadows** and **Highlights** wheels | Tint the dark and the light tones with a hue, at the chosen strength, without changing their brightness |
| **Balance** | Moves the point where the shadow tint gives way to the highlight tint; positive values give the highlight tint more of the photo |

Turning the conversion off keeps the mixer and tint settings, so you can compare color and black
and white. The panel's dot resets all of them.

## Check your work

| Control | What it does |
| --- | --- |
| **Hold for original**, or hold `\` | Shows the unedited photo while held |
| **Before / after split**, or `Y` | Splits the view; drag the divider to move it |
| **Clipping overlay**, or `J` | Marks pure black and pure white areas |
| **Scopes**, at the top of **Develop** | **Histogram**, **Waveform**, **Parade** and **Vectorscope**; **Pin scopes** fixes them above the panels |
| **More editor actions** > **Soft proof** | Previews the photo in a **Proof space**; **Show gamut warning** marks colors it cannot show |

## Undo and history

`Ctrl+Z` undoes and `Ctrl+Shift+Z` redoes. **Edit history**, next to **Copy edits** at the top of
**Develop**, lists every saved state. Expand an entry to see what changed, and click it to go back
to that state.

{: .warning }
Going back in **Edit history** deletes every entry after the one you pick. Make a
[virtual copy](#try-variations) first if you might want the later state again.

**Reset edits**, or `Ctrl+Shift+R`, returns the photo to its original state.

## Adjust part of the photo

**Masks** applies adjustments to one area. A mask is built from one or more shapes, and has its own
set of adjustments.

1. Press **New**, or **Create a mask** on a photo without masks.
1. Pick a shape:
   - **Linear gradient** and **Radial gradient** fade out from a line or an ellipse. Drag the
     handles on the photo.
   - **Brush** paints the area by hand. `[` and `]` change the size, `{` and `}` the hardness, and
     **Erase** takes paint away.
   - **Polygon** follows straight edges. Click to place corners and click the first one, or press
     `Enter`, to close it.
   - **Luminance range** and **Color range** pick areas by brightness or by a color you sample.
   - Under **AI**, **Subject**, **Background**, **People**, **Sky**, **Depth** and **Scene** find
     the area for you. **Click to select** and **Box select** find an object you click or draw a box
     around. These need a model that an administrator installs; see
     [mask models](administration.md#mask-models).
1. Set the adjustments below the mask list. **Amount** fades the whole mask in or out.

To refine a mask, open **Refine** on it and use **Add**, **Subtract** or **Intersect** with
another shape. For example, a **Sky** mask with a **Linear gradient** intersected keeps only the
top of the sky. **Invert** edits everything outside the shapes instead. An AI click mask also has
**Click refine**: press **Add** or **Remove** and click the photo to grow or trim it, then **Done**.

`O` shows or hides the colored mask overlay. Masks lower in the list are applied on top of the
ones above them, and **Duplicate layer** starts a new mask from an existing one.

## Remove spots and distractions

1. Open **Retouch** and pick **Heal**, which blends the source's texture into the spot, or
   **Clone**, which copies the source exactly.
1. Hold `Alt` and click a clean area to set the source.
1. Paint over the spot. The source follows the brush at the same offset.

Drag a stroke's green circle to move its source, and set **Size**, **Hardness** and **Opacity**
for the next stroke. `H` and `C` switch between heal and clone.

## Crop and straighten

**Geometry** shows the whole photo while you work, and applies the result when you switch to
another tool.

- **Crop**: drag the frame, or choose an **Aspect Ratio** such as **Original**, **Free**, **1:1** or
  **3:2**.
- **Angle** straightens the horizon. **Rotate left 90°** and **Rotate right 90°** turn the photo,
  and **Flip Horizontal** and **Flip Vertical** mirror it.
- **Vertical** and **Horizontal** correct converging lines, and **Aspect** restores proportions
  afterwards. **Corner handles**, or `Shift+P`, lets you drag the four corners onto a shape that
  should be rectangular instead.
- **Reset Transform** undoes everything in this tool.

## Reuse edits

**Copy edits** (`Ctrl+Shift+C`) opens **Copy settings**, where you pick what to copy: **Basic**,
**Tone**, **Color**, **Detail**, **Lens Corrections**, **Effects**, **Geometry & crop**, **Masks**
and **Retouch**. The last three start unchecked, because a crop or a mask rarely fits another
photo. Then open another photo and press **Paste edits** (`Ctrl+Shift+V`). To paste onto many
photos at once, use [**Edit and export selected**](export.md#export-many-photos).

A preset keeps a look for later. In **Presets**, press **Save current as preset**, give it a name
and an optional group, and **Save**. To apply one, pick it and press the **Apply** button under
it. The **Geometry & crop** and **Masks** checkboxes decide whether those come along.

**Amount** sets how strongly the preset applies, from 0% to 200%. At 100% you get the preset as
saved. Lower values move each slider part of the way from its default toward the preset, and
higher values push it further in the same direction, up to the slider's limit. Curves scale the
same way, starting from a straight line, and a LUT's own amount scales too. Choices rather
than amounts, such as the camera profile, the LUT file, sharpening and lens corrections, come
from the preset as saved. At 0% the look returns to defaults. Geometry and masks are never
scaled. Double-click the slider to go back to 100%.

**Export all** downloads every preset as one `.json` file, and the download button beside
**Rename** exports only the selected one. **Import presets** reads such a file and adds its
presets to your library. A preset whose name is already in your library is skipped, so
importing the same file twice adds nothing.

## Try variations

**Versions** lists the original and its virtual copies. **Create virtual copy**, or `Ctrl+'`,
starts a new version with its own edits and history. Rename or delete a version from its row.
Culling and copies are covered in [browse and cull](cull.md#virtual-copies).
