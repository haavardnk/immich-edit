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
on opens by itself. The search box at the top searches your library, and the **Search by** buttons
above the results pick what it matches, as in Immich:

- **Context** uses Immich's smart search, so a description such as `red car at night` works. When
  Immich's machine learning is down, it falls back to searching filenames.
- **File name** matches part of the original filename.
- **Description** matches part of the description set in Immich.
- **Full path or folder** matches part of the original file path, such as a folder name.
- **OCR** finds text Immich read in the photo.

A query that looks like a filename, such as `DSC00195` or `IMG_1234.jpg`, starts in **File name**;
anything else starts in **Context**. **Context** and **OCR** appear only when the Immich server has
smart search and OCR turned on; without smart search every query starts in **File name**. With no
results, **Context** offers **Search filenames instead**.

**Filters**, or `Shift+F`, opens the filter panel on the right of the grid; its close button or
`Shift+F` again hides it, and the **Filters** button stays hidden while the panel is open. The panel
groups its filters into **Library** (**Visibility**, **Rejected**, **Favorites only**), **Albums,
people & tags** (**Albums** with **Not in any album**, **People** with **No people**, **Tags** with
**Untagged**), **Rating & label**, **Date** (**Taken after**, **Taken before**, **Added after**,
**Added before**), **Camera** (**Make**, **Model**, **Lens**), **Place** (**Country**, **State**,
**City**) and **File** (**Filename**, **Min size (MB)**, **Max size (MB)**). Search leaves out
**Filename**. Sections start folded; click a section title to open or fold it. A section with an
active filter shows a reset button that clears only that section, and **Reset** at the top of the
panel clears every filter. The panel remembers whether it was open and which sections you left
open. **Visibility** switches between **Timeline**, **Archived** and **Hidden**; folders and
**Edited** show timeline photos and do not offer it. **Rating** picks **Any**, **Unrated** or a star
count. On Immich 3.2 and later, the button before the stars switches between `≥` (at least, the
default), `≤` (at most) and `=` (exactly); like Lightroom, `≤` also keeps unrated photos. Older
Immich versions match the exact star count. **Rejected** shows rejects, hides them or shows only
them; **Only** lets you review everything you rejected before
deleting it in Immich. **Not in any album** keeps photos that no album holds yet, so you can find
what still needs filing; albums, folders and **Edited** do not offer it. **Untagged** keeps photos
with no tags at all, and on Immich 3.2 and later **No people** keeps photos where Immich found no
faces. Color labels and reject marks are tags too, so a labelled or rejected photo is not untagged.
Tag pages leave out **Untagged**, people pages leave out **No people**, context search leaves
out **No people**, and folders and **Edited** offer neither.
**Min size (MB)** and **Max size (MB)** keep photos whose original file is at least or at most that
size, where 1 MB is 1,000,000 bytes. A photo with no recorded size never matches a size range.
Folders and **Edited** offer them on every Immich version, other views need Immich 3.2, and
context search does not offer them.
**Make**, **Model**, **Lens**, **Country**, **State** and **City** list the values Immich knows
from your timeline photos. Picking a make narrows the models and lenses on offer, and picking a
model narrows the lenses; a country narrows the states and cities the same way, and a state the
cities. A choice the new list no longer contains is dropped. On Immich 3.2 and later each picker
takes several values and matches any of them; older versions match one value.
When some photos lack the value, the list ends with **Unknown**, which finds those photos and
cannot be combined with other values.
**Albums** lists your albums, **People** the named people Immich shows and **Tags** your tags by
their full path; immich-edit's own label and reject tags stay out of it. Each takes several values,
and by default a photo must match every pick. On Immich 3.2 and later, **All of**, **Any of** and
**None of** under a picker switch to photos with at least one of them or none of them; older
versions match all of them. An album's, person's or tag's page leaves its own entry out of the
list, and folders and **Edited** do not offer these filters.
The sort menu next to **Filters** picks **Date taken** or **Date added** to Immich, and
**Newest first** or **Oldest first**. On Immich 3.2 and later it also offers **Rating**, with
**Highest first** or **Lowest first**, and **Size**, with **Largest first** or **Smallest first**.
Immich sorts the whole result, so the best or biggest photos lead every page, not only the loaded
ones. Unrated photos, and photos with no recorded size, come last in both orders, and photos with
the same rating keep no particular order. Each view remembers its own order, so **Date added** with
**Newest first** shows your latest uploads. **Reset filters and sort** clears both.
Each active filter shows as a chip under the header; its **Remove** button clears only that filter.
Filters live in the page address, so a reload keeps them, Back and Forward step through your
filter changes, and a bookmark or shared link opens the same filtered view. Typing a filename or
changing a date that is already set updates the address without adding a Back step. Each album,
folder, person or tag starts unfiltered, as does the **Photos** link in the sidebar. The sort order
is not part of the address; each view remembers its own.
The count beside the title is the number of photos in the view, followed by how many of the loaded
ones **Label** and **Rejected** are hiding because you changed them after they loaded. Before Immich
3.2, Immich cannot leave photos out by tag, so **Hide** rejects and **No label** filter the loaded
photos instead and the count still includes the ones they hide. Context search works the same
way on every Immich version, and matches only exact star counts.
The **S**, **M**, **L** and **XL** buttons set the thumbnail size. The **Thumbnail info** button
beside them, or `Shift+I`, cycles what a thumbnail shows without hovering: nothing at all, the
favorite, reject, color label, rating and copy badges, or those plus the filename and capture date.
Hovering a thumbnail shows five stars above its filename; click one to rate that photo, or the
lit star again to clear it. With filename and date showing, the stars stay visible.

## Look at photos

Clicking a photo opens it in the [editor](edit.md). To look without editing, hover it and press
**Quick review**, or select it and press `E`. That opens the loupe: one photo at full size, with the filmstrip
below. Arrow keys move through the photos, `Z` zooms, and `I` opens the info panel, which stays
open until you close it, reloads included. The loupe, the editor and their filmstrips load the rest
of the view as you reach the end of what is loaded, so the arrow keys walk the whole album or search
result. When you close the loupe or go back from the editor, the grid scrolls to the photo you ended
on if it is off screen.

`Ctrl` or `Cmd` with the mouse wheel zooms at the pointer in the loupe, compare and survey, the
wheel alone pans a zoomed photo, and a plain wheel over a filmstrip scrolls it.
In the loupe, compare, survey and the editor, `Shift` with an arrow key nudges a zoomed photo by a
tenth of the view. In the loupe, `Home` and `End` jump to the first and last photo, loading the
rest of the view first when needed.

`Z` zooms to the faces Immich found, largest first, and back to fit after the last one. Without
faces it zooms to the sharpest area. It zooms to the level you last picked, 1:1 until you change
it. Zoom percentages are percentages of the original, so 100% shows one camera pixel on one screen
pixel.

## Rate, reject and label

These keys work in the grid, the loupe, compare, survey and the editor. In the grid they act on
the selected photos; the arrow keys select one:

| Key | Does |
| --- | --- |
| `1` to `5` | Set the rating; the same key again clears it |
| `0` | Clear the rating |
| `Shift` + `0` to `5` | Set the rating and move to the next photo, whatever **Auto-advance** says. In the grid, one selected photo hands its selection to the next one; with several selected, all get the rating and the selection stays |
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
never deletes a photo or moves it to the trash; set **Rejected** to **Hide** to filter rejects out,
or to **Only** to find them and delete them in Immich later.

Photos another Immich user owns, such as a partner's photos in your timeline or another user's
photos in a shared album, follow Immich's rules: the rating, favorite, reject, label and tag
controls stay visible but are disabled, and hovering them says why. The keys above do nothing on
them. You can still add them to albums, edit them and export the result as a new photo of your
own. When Immich refuses a change for another reason, the message says what Immich answered.

## Choose between similar shots

In the grid, keys act on the selected photos, so select photos first:

- The arrow keys, `Home`, `End`, `Page Up` and `Page Down` select one photo. The first press
  selects the photo you last had open.
- `Shift` with those keys grows or shrinks the selection.
- `Ctrl`-click, or `Cmd`-click on macOS, adds or removes one photo.
- `Shift`-click selects everything between the last selected photo and this one.
- The circle in a thumbnail's corner also toggles it. Once anything is selected, a plain click
  toggles too.
- **Select all**, or `Ctrl+A`, loads and selects every photo in the view.

With two photos selected, press `C` or **Compare selected**. The two sit side by side, and zoom and
pan move together until you press `Y`. `Alt+←` or `Alt+→` (`⌥←` or `⌥→` on a Mac) swaps the focused
side for the previous or next photo, which is the quickest way to hold a winner and step through the
rest, even while zoomed in.
The left photo is labelled **Select** and the other **Candidate**. When the candidate is focused,
its **Make select** button, or `Enter`, moves it to the left.

With up to nine selected, press `N` or **Survey selected** to see them all at once. Drop the weaker
ones with `Backspace`, or the **Drop from survey** button that shows when you hover a photo, until
the keepers are left, or press `Enter` to keep only the focused photo.
`Esc` goes back to the loupe, and the survivors are selected when you return to the grid.

Selecting belongs to the grid. In the loupe, `C` compares the photo on screen with the next one
and `N` surveys it with the next five.

The loupe's **View mode** button switches between **Single photo**, **Compare** and **Survey** as
well. In compare and survey, the bar under the photos has buttons for the same pane actions:
**Sync zoom and pan**, **Make select** in compare or **Keep only this photo** in survey,
and **Drop this photo**. `Shift+T` hides or shows the filmstrip.
Right-click a filmstrip thumbnail for **Open in editor** and **Add to compare**, which adds or
removes a pane in compare and survey.

## A culling pass

1. Open the album, folder or day, and set **Rejected** to **Hide**.
1. Press `E` on the first photo. Step through with `→`, pressing `X` on misses and a rating on
   keepers.
1. At a burst, select the frames and press `N`. Drop frames until one is left, and rate it.
1. Under **Rating**, keep `≥` and pick the lowest star count you want to edit, such as three
   stars (Immich 3.2 and later). What is left is your edit list.

## Act on a selection

With photos selected, the selection bar shows one heart, one star row, one reject button and the
color label. Each shows what the selection already is: filled when every photo has it, faded when
only some do. A click sets every photo the same way, so **Favorite** on a mixed selection favorites
all of them instead of flipping each one, and a star sets that rating on all. Click the lit star
again to clear the rating. The bar also has **Tags**, **Albums**, **Create virtual copy**, and
**Edit and export selected**. **Albums** adds the selection to existing Immich albums; inside an
album it also offers **Remove from** that album, which drops the photos from the grid while they
stay in Immich. **Edit and export selected** pastes edits, applies a preset or exports every selected
photo in one go; see [export many photos](export.md#export-many-photos). When the selection holds a
photo another Immich user owns, the favorite, rating, reject, label and **Tags** controls are
disabled, and hovering them says why. `Shift+B` moves keyboard focus from the grid into the
selection bar.

With one photo selected, the bar shows its filename in place of the count and an **Open in
editor** button. With two it offers **Compare selected**, and with three or more **Survey
selected**. **Select all** next to the count loads and selects every photo in the view.

`Ctrl+Shift+C` (`⌘⇧C` on a Mac) copies the edits of the selected photo in the grid, or of the photo
on screen in the loupe, and asks which settings to take, remembering your last choice.
`Ctrl+Shift+V` pastes them onto every selected photo as a background job; in the loupe it pastes
onto the photo on screen.

## Virtual copies

A virtual copy is a second, independent set of edits on the same original, such as a black and
white version next to the color one. It takes no space in Immich until you export it. Create one
with **Create virtual copy** or `Ctrl+'`. It shows up next to the original with a `Copy` badge.

Ratings, favorites, tags and reject marks belong to the Immich photo, so every copy shares them.
Edits, masks, history and exports belong to one copy. Delete a copy from its thumbnail, or from the
[**Versions**](edit.md#try-variations) panel in the editor.

The full key list is on [keyboard shortcuts](shortcuts.md).
