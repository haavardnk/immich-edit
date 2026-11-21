---
layout: default
title: Browse and cull
parent: Use the editor
nav_order: 1
permalink: /cull/
---

# Browse and cull

immich-edit reads your library straight from Immich. Nothing is imported or copied, so a photo is
here as soon as Immich has it. Keys are listed on [keyboard shortcuts](shortcuts.md).

## Find photos

The sidebar has **Photos**, **People**, **Favorites**, **Albums**, **Tags**, **Folders** and
**Edited** (the photos you changed here). **People**, **Albums**, **Tags** and **Folders** open
their overview page; the arrow beside each lists them in the sidebar. The search box searches your
library; **Search by** picks **Context** (smart search), **File name**, **Description**, **Full path
or folder** or **OCR**.

**Album actions** (⋯ on an album, or right-click it) renames an album, edits its description,
deletes it or leaves one shared with you. The photos stay in your library.

**Filters** (`Shift+F`) opens the filter panel: **Library**, **Rating & label**, **People**,
**Albums**, **Tags**, **Date**, **Camera**, **Place** and **File**. **Reset** clears them all, and
each active filter shows as a chip you can remove. Filters live in the page address, so a reload
or a shared link keeps them. The sort menu beside it orders by date, and on Immich 3.2 and later
by **Rating** or **Size**. **S** to **XL** set the thumbnail size.

Sorted by **Date taken**, the grid is grouped by day, and the scrubber at the right edge jumps to
any month. **Save view** in the filter panel keeps the page, filters and sort under a name in your
account.

## Look at photos

Clicking a photo opens the loupe; hover a photo and press **Edit**, or press **Edit** in the loupe,
to open the [editor](edit.md). In the loupe the arrow keys move through photos, `Z` zooms, and `I`
opens the info panel with the description, camera, place, albums and tags. Hold `W` over a photo
in the grid to peek at it.

## Rate, reject and label

`1` to `5` rate, `P` favorites, `X` rejects and `6` to `9` set a color label, in the grid, loupe
and editor. **Auto-advance** in the loupe toolbar moves to the next photo after each mark. Ratings,
favorites and tags are written to Immich; reject and labels are the Immich tags
`immich-edit/reject` and `immich-edit/label/<color>`. Reject deletes nothing; filter rejects with
**Rejected** and [trash them](#delete-photos) when you are done. Photos another Immich user owns
cannot be marked.

## Choose between similar shots

Select photos with a click, `Shift`-click, `Ctrl`-click or by dragging a box from a gap. With two
selected, press `C` (**Compare selected**) to see them side by side; with up to nine, press `N`
(**Survey selected**) and drop the weaker ones with `Backspace` until the keepers are left.

## A culling pass

1. Open the album or folder and set **Rejected** to **Hide**.
1. Press `E` on the first photo and step through with `→`, pressing `X` on misses and a rating on
   keepers.
1. At a burst, select the frames, press `N` and drop frames until one is left.
1. Under **Rating & label**, keep **At least** and pick your minimum rating. That is your edit
   list.

## Act on a selection

The selection bar sets favorite, rating, reject and label on every selected photo, and has
**Create virtual copy**, **Edits** (apply a preset, paste or reset), **Tags**, **Albums**,
**Export selected photos** (see [export many photos](export.md#export-many-photos)) and **Move to
Immich trash**. `Ctrl+C` and `Ctrl+V` copy one photo's edits and paste them onto the selection.

## On a phone or tablet

Phones and tablets in portrait get a tab bar with **Photos**, **Albums**, **Search** and
**Library**; **View & filters** holds the sort and filters. Tap a photo to open it, hold to start
selecting, and swipe in the loupe. **Settings** > **Preferences** > **Layout** picks a layout
yourself; see [this device](administration.md#this-device).

## Delete photos

**Move to Immich trash** sends photos you own to the Immich trash, after asking; **Undo** brings
them back. To trash every reject in a view, set **Rejected** to **Only** and press **Move rejected
to trash**. Deleting a virtual copy removes it from immich-edit and cannot be undone.

{: .warning }
immich-edit deletes nothing while the trash is turned off in Immich, because Immich would then
remove the photos for good. Signed in with an Immich API key, the key needs the `asset.delete`
permission.

## Virtual copies

A virtual copy is a second set of edits on the same original, such as a black and white version.
Create one with **Create virtual copy** or `Ctrl+'`; it takes no space in Immich until you export
it. Ratings and tags are shared with the original, edits are not. Manage copies under
[**Versions**](edit.md#try-variations) in the editor.
