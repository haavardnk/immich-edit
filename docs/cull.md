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

**Tags** shows your tags as a tree, in the sidebar and on its page: the arrow beside a tag opens
its sub-tags, the filter keeps each match under its parents, and the page sorts by name or photo
count. A tag's count includes its sub-tags, and its page shows its path and its sub-tags. The
`immich-edit/` tags behind rejects and labels are not listed.

**Create tag** (+) adds a tag, and **Tag actions** (⋯, a right-click or a long press) renames a
tag, changes its color, adds a sub-tag or deletes it. Deleting removes the tag and its sub-tags
from every photo; the photos stay in your library. Renaming needs Immich 3.3 or later.

**Album actions** (⋯ on an album, or right-click it) renames an album, edits its description,
deletes it or leaves one shared with you. The photos stay in your library. In an album you own or
edit, **Set as album cover** in the loupe's ⋯ menu makes the photo on screen its cover.

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
Immich trash**. On an album you own or edit it also has **Remove from album**, with **Undo**.
`Ctrl+C` and `Ctrl+V` copy one photo's edits and paste them onto the selection.

## On a phone or tablet

Phones and tablets in portrait get a tab bar with **Photos**, **Albums**, **Search** and
**Library**; **View & filters** holds the sort and filters. Tap a photo to open it, hold to start
selecting, and swipe in the loupe. **Settings** > **Preferences** > **Layout** picks a layout
yourself; see [this device](administration.md#this-device).

## Places and the map

**Map** in the library shows every photo that has a location, grouped by area until you zoom in.
The panel beside it lists the photos in view. Click a group to zoom to it and list only its
photos; double-click a group or photo to select it. **No location** lists photos without one.
**Favorites** and **Any date** narrow the map and both lists, and **Fit all photos** zooms out to
all of them. The panel button next to it hides or shows the panel, and its edge can be dragged
to resize it. On a phone, the pill at the bottom opens the list in a sheet; tapping a group opens
it too, and moving the map closes it.

Drag photos from the panel onto the map to place them there; dragging a selected photo places the
whole selection. On a phone, press and hold a photo in the sheet and drag it up out of the sheet;
the sheet steps aside and you place the photo, and anything else selected, on the map. To move a
photo or a group on the map, press and hold it, then drag; press Escape to put it back. A quick
drag on a marker pans the map instead. Every drop asks before it changes anything: **Move** or
**Place** confirms, and **Cancel** or Escape leaves the photos where they were. **Undo** in the
toast puts moved photos back. Selected
photos can also be placed with **Location** in the selection bar.

The photo **Info** panel shows a small map for a photo with a location. Click it to open
**Map** there. **Set location** or **Edit location** opens a map where you search for a place or
move the pin; on touch, move the map under the pin. **Copy location** and **Paste location** carry
a location between photos. **Location** in the selection bar sets it for every selected photo you
own. **Undo** in the toast puts the old locations back.

{: .note }
Immich can change a location but cannot remove one, so **Undo** leaves the new location on photos
that had none.

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
