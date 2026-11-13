---
layout: default
title: Keyboard shortcuts
parent: Use the editor
nav_order: 4
permalink: /shortcuts/
---

<!-- Generated from web/src/lib/shortcuts/table.ts. Run `npm run docs:shortcuts` in web/ to update. -->

# Keyboard shortcuts

Press `?` anywhere in immich-edit to open the same list as a searchable dialog. On macOS, press
Command wherever a shortcut below says `Ctrl`.

The keys below are the defaults. In that dialog, click a key and press a new one to change it; if
the new key already runs something else, you can swap the two. Ranges, arrow keys, `Esc` and
pointer gestures stay fixed. Your shortcuts are saved to your account and work on every device
with a keyboard.

`Shift` with a letter opens a tool, panel or strip, a plain letter acts on the photo, and `Ctrl`
runs a command. Symbol keys such as `[`, `{` and `\` work however your keyboard layout types
them, including with `Option` or `AltGr`.

Shortcuts do not run while focus is in a text field or another typing control.

## General

| Keys            | Action                  | Available in                   |
| --------------- | ----------------------- | ------------------------------ |
| `?` / `Shift+/` | Show keyboard shortcuts | Anywhere                       |
| `G`             | Back to the grid        | Loupe, Compare, Survey, Editor |

## Culling

| Keys                                                                  | Action                                             | Available in                         |
| --------------------------------------------------------------------- | -------------------------------------------------- | ------------------------------------ |
| `0` / `1` / `2` / `3` / `4` / `5`                                     | Set, toggle or clear the rating                    | Grid, Loupe, Compare, Survey, Editor |
| `Shift+0` / `Shift+1` / `Shift+2` / `Shift+3` / `Shift+4` / `Shift+5` | Set the rating and move to the next photo          | Grid, Loupe, Compare, Survey         |
| `P`                                                                   | Toggle favorite                                    | Grid, Loupe, Compare, Survey, Editor |
| `X`                                                                   | Toggle reject                                      | Grid, Loupe, Compare, Survey, Editor |
| `U`                                                                   | Clear favorite and reject                          | Grid, Loupe, Compare, Survey, Editor |
| `6` / `7` / `8` / `9`                                                 | Set or clear the red, yellow, green or blue label  | Grid, Loupe, Compare, Survey, Editor |
| `Del` / `Backspace`                                                   | Move to the Immich trash, or delete a virtual copy | Grid, Loupe                          |

## Grid

| Keys                                                                                                     | Action                                                           | Available in |
| -------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------- | ------------ |
| `←` / `→` / `↑` / `↓`                                                                                    | Select the next photo in that direction                          | Grid         |
| `Shift+←` / `Shift+→` / `Shift+↑` / `Shift+↓` / `Shift+Home` / `Shift+End` / `Shift+PgUp` / `Shift+PgDn` | Extend the selection, also with Home, End, Page Up and Page Down | Grid         |
| `Home` / `End`                                                                                           | Select the first or last photo                                   | Grid         |
| `PgUp` / `PgDn`                                                                                          | Jump a page                                                      | Grid         |
| `-` / `_` / `=` / `+`                                                                                    | Thumbnail size                                                   | Grid         |
| `Ctrl scroll · pinch`                                                                                    | Change the thumbnail size around the pointer                     | Grid         |
| `I`                                                                                                      | Cycle thumbnail info: hover only, badges, name and date          | Grid         |
| `Shift+F`                                                                                                | Show or hide the filter panel                                    | Grid         |
| `Ctrl+A`                                                                                                 | Load and select every photo                                      | Grid         |
| `Esc`                                                                                                    | Clear the selection                                              | Grid         |
| `Drag from a gap`                                                                                        | Box-select the photos it touches                                 | Grid         |
| `Shift or Ctrl + drag from a gap`                                                                        | Add the boxed photos to the selection                            | Grid         |
| `Shift+B`                                                                                                | Focus the selection actions                                      | Grid         |
| `E` / `Space`                                                                                            | Open the loupe                                                   | Grid         |
| `W`                                                                                                      | Hold to peek at the photo                                        | Grid         |
| `D` / `Enter`                                                                                            | Open the editor                                                  | Grid, Loupe  |
| `C`                                                                                                      | Compare the selection, or this photo and the next                | Grid, Loupe  |
| `N`                                                                                                      | Survey the selection, or this photo and the next five            | Grid, Loupe  |

## Loupe

| Keys                                                                          | Action                           | Available in                   |
| ----------------------------------------------------------------------------- | -------------------------------- | ------------------------------ |
| `←` / `→`                                                                     | Previous / next photo            | Loupe                          |
| `Shift+←` / `Shift+→` / `Shift+↑` / `Shift+↓`                                 | Nudge a zoomed photo             | Loupe, Compare, Survey, Editor |
| `Home` / `End`                                                                | First / last photo               | Loupe                          |
| `Z` / `Space`                                                                 | Toggle zoom                      | Loupe, Compare, Survey, Editor |
| `I`                                                                           | Toggle the info panel            | Loupe, Compare, Survey, Editor |
| `Shift+T`                                                                     | Toggle the tags panel            | Loupe, Editor                  |
| `Alt+1` / `Alt+2` / `Alt+3` / `Alt+4` / `Alt+5` / `Alt+6` / `Alt+7` / `Alt+8` | Add one of the eight recent tags | Loupe, Compare, Survey         |
| `J`                                                                           | Toggle the clipping indicators   | Loupe, Compare, Survey, Editor |
| `Esc`                                                                         | Close the loupe                  | Loupe                          |

## Compare

| Keys                | Action                                  | Available in    |
| ------------------- | --------------------------------------- | --------------- |
| `←` / `→`           | Move focus between panes                | Compare         |
| `Tab` / `Shift+Tab` | Cycle focus between panes               | Compare, Survey |
| `Alt+←` / `Alt+→`   | Swap the focused pane for another photo | Compare, Survey |
| `Y`                 | Toggle synced zoom and pan              | Compare, Survey |
| `D`                 | Open the focused photo in the editor    | Compare, Survey |
| `Backspace` / `Del` | Drop the focused photo                  | Compare, Survey |
| `Enter`             | Make the focused photo the select       | Compare         |
| `E` / `Esc`         | Back to the loupe on the focused photo  | Compare         |

## Survey

| Keys                  | Action                                                                    | Available in |
| --------------------- | ------------------------------------------------------------------------- | ------------ |
| `←` / `→` / `↑` / `↓` | Move focus between panes                                                  | Survey       |
| `Enter`               | Keep only the focused photo                                               | Survey       |
| `E` / `Esc`           | Back to the loupe, selecting the survivors in the grid if you dropped any | Survey       |

## View

| Keys      | Action                         | Available in                                  |
| --------- | ------------------------------ | --------------------------------------------- |
| `F`       | Toggle fullscreen              | Loupe, Compare, Survey, Editor, Viewer window |
| `Shift+S` | Hide or show the filmstrip     | Loupe, Compare, Survey, Editor                |
| `S`       | Show or hide the scope overlay | Editor                                        |

## Loupe gestures

| Keys                                          | Action                                                          | Available in           |
| --------------------------------------------- | --------------------------------------------------------------- | ---------------------- |
| `Click the photo`                             | Zoom in at the pointer, or back to fit when zoomed              | Loupe, Compare, Survey |
| `Drag or scroll`                              | Pan a zoomed photo                                              | Loupe, Compare, Survey |
| `Ctrl + scroll`                               | Zoom at the pointer                                             | Loupe, Compare, Survey |
| `Alt + click or drag`                         | Zoom or pan only the focused pane while zoom and pan are synced | Compare, Survey        |
| `Shift or Ctrl + click a filmstrip thumbnail` | Add or remove a compare pane                                    | Loupe, Compare, Survey |

## Editor

| Keys           | Action                                                | Available in        |
| -------------- | ----------------------------------------------------- | ------------------- |
| `←` / `→`      | Previous / next photo                                 | Editor              |
| `Ctrl+Z`       | Undo                                                  | Editor              |
| `Ctrl+Shift+Z` | Redo                                                  | Editor              |
| `Shift+D`      | Open Develop                                          | Editor              |
| `Shift+R`      | Open Crop                                             | Editor              |
| `Shift+Q`      | Open Retouch                                          | Editor              |
| `Shift+M`      | Open Masks                                            | Editor              |
| `Shift+P`      | Show or hide the presets strip                        | Editor              |
| `Ctrl+'`       | Create a virtual copy                                 | Grid, Loupe, Editor |
| `Ctrl+Shift+S` | Save a snapshot of the current edits                  | Editor              |
| `Y`            | Cycle compare: split, side by side, off               | Editor              |
| `\` / `<`      | Hold to view the original                             | Editor              |
| `Tab`          | Hide or show the side panels                          | Editor              |
| `Shift+Tab`    | Hide or show every panel                              | Editor              |
| `W`            | Pick white balance from a neutral area                | Editor              |
| `T`            | Adjust HSL, Curves, B&W or Color Grading on the photo | Editor              |
| `Ctrl+U`       | Auto adjust                                           | Editor              |
| `Ctrl+K`       | Search settings, presets and actions                  | Editor              |
| `Shift+Enter`  | Pin or unpin the highlighted palette setting          | Editor              |
| `Ctrl+Shift+R` | Reset develop settings, or geometry on the Crop tab   | Editor              |
| `Ctrl+C`       | Copy edits                                            | Editor, Grid, Loupe |
| `Ctrl+V`       | Paste edits                                           | Editor, Grid, Loupe |
| `Ctrl+Shift+V` | Paste only some settings                              | Editor, Grid, Loupe |
| `Shift+E`      | Open Export                                           | Editor              |
| `Esc`          | Step out of the active tool, panel or fullscreen      | Editor              |

## Sliders

| Keys                                | Action                                                                          | Available in |
| ----------------------------------- | ------------------------------------------------------------------------------- | ------------ |
| `Shift+← / Shift+→`                 | Move a focused slider ten steps                                                 | Editor       |
| `Alt+← / Alt+→`                     | Move a focused slider by its finest step                                        | Editor       |
| `Wheel, Shift + wheel, Alt + wheel` | Adjust the slider under the pointer once it has rested there, ten steps or fine | Editor       |
| `Drag the label or track`           | Adjust a slider from its current value; move away from the track to slow down   | Editor       |
| `Click the value`                   | Type a value, or +5, x1.5 or /2 to change the current one                       | Editor       |
| `Double-click the label`            | Reset one slider                                                                | Editor       |
| `Alt + drag`                        | Preview the mask or radius a slider drives while you drag it                    | Editor       |
| `Shift + click the reset button`    | Reset every band or channel at once, on HSL and Curves                          | Editor       |
| `Alt + drag a curve point`          | Move a curve point at a quarter of the speed                                    | Editor       |
| `Alt + drag on the photo`           | Adjust hue while dragging on the photo                                          | Editor       |
| `Shift + drag on the photo`         | Adjust luminance while dragging on the photo                                    | Editor       |

## Crop

| Keys                                         | Action                                                                              | Available in |
| -------------------------------------------- | ----------------------------------------------------------------------------------- | ------------ |
| `A`                                          | Draw a line along the horizon to level the photo                                    | Crop         |
| `Shift + drag or right-drag inside the crop` | Draw a straighten line without arming the tool                                      | Crop         |
| `O`                                          | Cycle the crop guide: thirds, golden ratio, spiral, diagonals, triangle, grid, none | Crop         |
| `Shift+O`                                    | Turn the golden spiral or triangle guide                                            | Crop         |
| `Alt + drag a crop handle`                   | Resize the crop about its centre                                                    | Crop         |
| `Shift + drag a crop handle`                 | Keep the current shape of a free crop while resizing                                | Crop         |
| `Ctrl + scroll over the crop`                | Scale the crop about its centre                                                     | Crop         |
| `Ctrl + drag inside the crop`                | Move the photo under a fixed crop                                                   | Crop         |

## Masks

| Keys                                                      | Action                                                                  | Available in   |
| --------------------------------------------------------- | ----------------------------------------------------------------------- | -------------- |
| `Backspace` / `Del`                                       | Delete the selected shape, or undo the last polygon corner              | Masks          |
| `K`                                                       | Add a brush mask layer                                                  | Editor         |
| `R`                                                       | Add a radial mask layer                                                 | Editor         |
| `L`                                                       | Add a linear mask layer                                                 | Editor         |
| `↑` / `↓` / `Shift+←` / `Shift+→` / `Shift+↑` / `Shift+↓` | Nudge the selected shape one pixel, ten with Shift                      | Masks          |
| `Click the dot between two corners`                       | Add a polygon corner                                                    | Masks          |
| `Double-click the corner`                                 | Remove a polygon corner                                                 | Masks          |
| `Shift + drag a radial size handle`                       | Keep a radial round while resizing it                                   | Masks          |
| `Shift + drag a linear end`                               | Snap a linear gradient to 45° steps                                     | Masks          |
| `Shift + drag a polygon corner`                           | Move a polygon corner straight across or straight up and down           | Masks          |
| `O`                                                       | Toggle the mask overlay                                                 | Masks          |
| `Shift+O`                                                 | Cycle the mask overlay: color, color on black and white, white on black | Masks          |
| `Esc`                                                     | Cancel drawing, click select, box select or the eyedropper              | Masks          |
| `Enter`                                                   | Close the polygon you are drawing                                       | Masks          |
| `[` / `]`                                                 | Smaller / larger brush                                                  | Masks, Retouch |
| `{` / `}`                                                 | Softer / harder brush                                                   | Masks, Retouch |
| `Alt + scroll`                                            | Resize the brush over the photo                                         | Masks, Retouch |
| `Alt + Shift + scroll`                                    | Change the brush hardness over the photo                                | Masks, Retouch |

## Retouch

| Keys                  | Action                            | Available in |
| --------------------- | --------------------------------- | ------------ |
| `H`                   | Heal mode                         | Retouch      |
| `C`                   | Clone mode                        | Retouch      |
| `Backspace` / `Del`   | Delete the selected stroke        | Retouch      |
| `Esc`                 | Deselect the current stroke       | Retouch      |
| `Alt + click`         | Set the source point              | Retouch      |
| `Drag the green ring` | Move the selected stroke's source | Retouch      |
