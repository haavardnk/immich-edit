---
layout: default
title: Browse and cull
parent: Use the editor
nav_order: 1
permalink: /cull/
---

# Browse and cull

immich-edit reads your library straight from Immich. Nothing is imported or copied, so a photo is
here as soon as Immich has it.

## Find photos

The sidebar has **Photos**, **People**, **Favorites**, **Albums**, **Tags**, **Folders** and
**Edited**. **Edited** lists the photos you have changed in immich-edit. **People**, **Albums**,
**Tags** and **Folders** remember whether you left them open, and the one holding the page you are
on opens by itself. The search box at the top uses Immich's smart search, so a description such as
`red car at night` works. When Immich's machine learning is down, it falls back to searching
filenames.

**Filters** narrows the current view by **Visibility**, **Rating**, **Label**, **Favorites only**,
**Exclude rejected**, **Filename**, **Taken after** and **Taken before**. **Visibility** switches
between **Timeline**, **Archived** and **Hidden**. The sort button next to it flips between newest
and oldest first, and each view remembers its own order. **Reset filters and sort** clears both.
Each active filter shows as a chip under the header; its **Remove** button clears only that filter.
The count beside the title is the number of photos in the view, followed by how many of the loaded
ones **Label** and **Exclude rejected** are hiding.
The **S**, **M**, **L** and **XL** buttons set the thumbnail size. The **Thumbnail info** button
beside them, or `Shift+I`, cycles what a thumbnail shows without hovering: nothing at all, the
favorite, reject, color label, rating and copy badges, or those plus the filename and capture date.

## Look at photos

Clicking a photo opens it in the [editor](edit.md). To look without editing, hover it and press
**Quick review**, or press `E`. That opens the loupe: one photo at full size, with the filmstrip
below. Arrow keys move through the photos, `Z` zooms, and `I` opens the info panel, which stays
open until you close it, reloads included. The loupe, the editor and their filmstrips load the rest
of the view as you reach the end of what is loaded, so the arrow keys walk the whole album or search
result. When you close the loupe or go back from the editor, the grid scrolls to the photo you ended
on if it is off screen.

`Ctrl` or `Cmd` with the mouse wheel zooms at the pointer in the loupe, compare and survey, the
wheel alone pans a zoomed photo, and a plain wheel over a filmstrip scrolls it.

`Z` zooms to the faces Immich found, largest first, and back to fit after the last one. Without
faces it zooms to the sharpest area. It zooms to the level you last picked, 1:1 until you change
it. Zoom percentages are percentages of the original, so 100% shows one camera pixel on one screen
pixel.

## Rate, reject and label

These keys work in the grid, the loupe, compare, survey and the editor:

| Key | Does |
| --- | --- |
| `1` to `5` | Set the rating; the same key again clears it |
| `0` | Clear the rating |
| `P` or `F` | Toggle favorite |
| `X` | Toggle reject |
| `U` | Clear favorite and reject |
| `6` to `9` | Set the red, yellow, green or blue label; the same key again clears it |

A photo has at most one color label: red, yellow, green, blue or purple. Purple has no key; pick
it, or any other label, with the label button beside reject in the loupe, the editor and the
selection bar. Tiles and filmstrip thumbnails show the label as a colored dot, and the **Label**
filter shows one color, or only photos without a label.

The first time you rate, favorite, tag, reject or label in a browser, immich-edit asks before it
writes to Immich. Press **Sync to Immich** to allow it. Ratings, favorites and tags are stored in
Immich, so they show up there too. Reject adds the Immich tag `immich-edit/reject`, and a label
adds `immich-edit/label/red` and so on, so Immich searches and workflows can use them. immich-edit
never deletes a photo or moves it to the trash; filter rejects out with **Exclude rejected**, or
delete them in Immich later.

## Choose between similar shots

Select photos first:

- `Ctrl`-click, or `Cmd`-click on macOS, adds or removes one photo.
- `Shift`-click selects everything between the last selected photo and this one.
- The circle in a thumbnail's corner also toggles it. Once anything is selected, a plain click
  toggles too.
- **Select all**, or `Ctrl+A`, loads and selects every photo in the view.

With two photos selected, press `C` or **Compare selected**. The two sit side by side, and zoom and
pan move together until you press `Y`. `Shift+←` or `Shift+→` swaps the focused side for the
previous or next photo, which is the quickest way to hold a winner and step through the rest.

With up to nine selected, press `N` or **Survey selected** to see them all at once. Drop the weaker
ones with `Backspace` until the keepers are left, or press `Enter` to keep only the focused photo.
`Esc` goes back to the loupe with the survivors selected.

The loupe's **View mode** button switches between **Single photo**, **Compare** and **Survey** as
well. In compare and survey, the bar under the photos has buttons for the same pane actions:
**Sync zoom and pan**, **Promote to the left** in compare or **Keep only this photo** in survey,
and **Drop this photo**. `Shift+T` hides or shows the filmstrip.

## A culling pass

1. Open the album, folder or day, and turn on **Exclude rejected**.
1. Press `E` on the first photo. Step through with `→`, pressing `X` on misses and a rating on
   keepers.
1. At a burst, select the frames and press `N`. Drop frames until one is left, and rate it.
1. Set **Rating** to the lowest rating you want to edit. What is left is your edit list.

## Act on a selection

With photos selected, the selection bar shows one heart, one star row, one reject button and the
color label. Each shows what the selection already is: filled when every photo has it, faded when
only some do. A click sets every photo the same way, so **Favorite** on a mixed selection favorites
all of them instead of flipping each one, and a star sets that rating on all. Click the lit star
again to clear the rating. The bar also has **Tags**, **Create virtual copy**, and
**Edit and export selected**. The last one pastes edits, applies a preset or exports every selected
photo in one go; see [export many photos](export.md#export-many-photos). `Shift+B` moves keyboard
focus from the grid into the selection bar.

## Virtual copies

A virtual copy is a second, independent set of edits on the same original, such as a black and
white version next to the color one. It takes no space in Immich until you export it. Create one
with **Create virtual copy** or `Ctrl+'`. It shows up next to the original with a `Copy` badge.

Ratings, favorites, tags and reject marks belong to the Immich photo, so every copy shares them.
Edits, masks, history and exports belong to one copy. Delete a copy from its thumbnail, or from the
[**Versions**](edit.md#try-variations) panel in the editor.

The full key list is on [keyboard shortcuts](shortcuts.md).
