---
layout: default
title: Keyboard shortcuts
parent: Use the editor
nav_order: 4
permalink: /shortcuts/
---

<!-- Generated from web/src/lib/keybinds.ts. Run `npm run docs:shortcuts` in web/ to update. -->

# Keyboard shortcuts

Press `?` anywhere in immich-edit to open the same list as a searchable dialog. On macOS, press
Command wherever a shortcut below says `Ctrl`.

Shortcuts do not run while focus is in a text field or another typing control.

## General

| Keys            | Action                  | Available in                   |
| --------------- | ----------------------- | ------------------------------ |
| `?` / `Shift+/` | Show keyboard shortcuts | Anywhere                       |
| `G`             | Back to the grid        | Loupe, Compare, Survey, Editor |

## Culling

| Keys                                                                  | Action                                            | Available in                         |
| --------------------------------------------------------------------- | ------------------------------------------------- | ------------------------------------ |
| `0` / `1` / `2` / `3` / `4` / `5`                                     | Set, toggle or clear the rating                   | Grid, Loupe, Compare, Survey, Editor |
| `Shift+0` / `Shift+1` / `Shift+2` / `Shift+3` / `Shift+4` / `Shift+5` | Set the rating and move to the next photo         | Grid, Loupe, Compare, Survey         |
| `P` / `F`                                                             | Toggle favorite                                   | Grid, Loupe, Compare, Survey, Editor |
| `X`                                                                   | Toggle reject                                     | Grid, Loupe, Compare, Survey, Editor |
| `U`                                                                   | Clear favorite and reject                         | Grid, Loupe, Compare, Survey, Editor |
| `6` / `7` / `8` / `9`                                                 | Set or clear the red, yellow, green or blue label | Grid, Loupe, Compare, Survey, Editor |

## Grid

| Keys                                                                                                     | Action                                                           | Available in |
| -------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------- | ------------ |
| `←` / `→` / `↑` / `↓`                                                                                    | Select the next photo in that direction                          | Grid         |
| `Shift+←` / `Shift+→` / `Shift+↑` / `Shift+↓` / `Shift+Home` / `Shift+End` / `Shift+PgUp` / `Shift+PgDn` | Extend the selection, also with Home, End, Page Up and Page Down | Grid         |
| `Home` / `End`                                                                                           | Select the first or last photo                                   | Grid         |
| `PgUp` / `PgDn`                                                                                          | Jump a page                                                      | Grid         |
| `-` / `_` / `=` / `+`                                                                                    | Thumbnail size                                                   | Grid         |
| `Shift+I`                                                                                                | Cycle thumbnail info: hover only, badges, name and date          | Grid         |
| `Ctrl+A`                                                                                                 | Load and select every photo                                      | Grid         |
| `Esc`                                                                                                    | Clear the selection                                              | Grid         |
| `Shift+B`                                                                                                | Focus the selection actions                                      | Grid         |
| `E` / `Space`                                                                                            | Open the loupe                                                   | Grid         |
| `D` / `Enter`                                                                                            | Open the editor                                                  | Grid, Loupe  |
| `C`                                                                                                      | Compare the selected photos                                      | Grid, Loupe  |
| `N`                                                                                                      | Survey the selected photos                                       | Grid, Loupe  |

## Loupe

| Keys                                          | Action                               | Available in                   |
| --------------------------------------------- | ------------------------------------ | ------------------------------ |
| `←` / `→`                                     | Previous / next photo                | Loupe                          |
| `Shift+←` / `Shift+→` / `Shift+↑` / `Shift+↓` | Nudge a zoomed photo                 | Loupe, Compare, Survey, Editor |
| `Home` / `End`                                | First / last photo                   | Loupe                          |
| `Z` / `Space`                                 | Toggle zoom                          | Loupe, Compare, Survey, Editor |
| `I`                                           | Toggle the info panel                | Loupe, Compare, Survey, Editor |
| `T`                                           | Toggle the tags panel                | Loupe, Editor                  |
| `J`                                           | Toggle the clipping indicators       | Loupe, Compare, Survey, Editor |
| `Esc`                                         | Close the loupe                      | Loupe                          |
| `S`                                           | Select or deselect the focused photo | Loupe, Compare, Survey         |

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

| Keys                  | Action                                                        | Available in |
| --------------------- | ------------------------------------------------------------- | ------------ |
| `←` / `→` / `↑` / `↓` | Move focus between panes                                      | Survey       |
| `Enter`               | Keep only the focused photo                                   | Survey       |
| `E` / `Esc`           | Back to the loupe, selecting the survivors if you dropped any | Survey       |

## View

| Keys      | Action                     | Available in                   |
| --------- | -------------------------- | ------------------------------ |
| `Shift+F` | Toggle fullscreen          | Loupe, Compare, Survey, Editor |
| `Shift+T` | Hide or show the filmstrip | Loupe, Compare, Survey         |

## Loupe gestures

| Keys                                          | Action                                                          | Available in           |
| --------------------------------------------- | --------------------------------------------------------------- | ---------------------- |
| `Click the photo`                             | Zoom in at the pointer, or back to fit when zoomed              | Loupe, Compare, Survey |
| `Drag or scroll`                              | Pan a zoomed photo                                              | Loupe, Compare, Survey |
| `Ctrl + scroll`                               | Zoom at the pointer                                             | Loupe, Compare, Survey |
| `Alt + click or drag`                         | Zoom or pan only the focused pane while zoom and pan are synced | Compare, Survey        |
| `Shift or Ctrl + click a filmstrip thumbnail` | Add or remove a compare pane                                    | Loupe, Compare, Survey |

## Editor

| Keys           | Action                                           | Available in        |
| -------------- | ------------------------------------------------ | ------------------- |
| `←` / `→`      | Previous / next photo                            | Editor              |
| `Ctrl+Z`       | Undo                                             | Editor              |
| `Ctrl+Shift+Z` | Redo                                             | Editor              |
| `D`            | Open Develop                                     | Editor              |
| `R`            | Open Geometry                                    | Editor              |
| `Q`            | Open Retouch                                     | Editor              |
| `M`            | Open Masks                                       | Editor              |
| `Ctrl+'`       | Create a virtual copy                            | Grid, Loupe, Editor |
| `Shift+P`      | Toggle the perspective corner handles            | Editor              |
| `Y`            | Toggle the before / after split                  | Editor              |
| `\`            | Hold to view the original                        | Editor              |
| `Tab`          | Hide or show the side panels                     | Editor              |
| `Shift+Tab`    | Hide or show every panel                         | Editor              |
| `Ctrl+U`       | Auto adjust tone                                 | Editor              |
| `Ctrl+Shift+R` | Reset develop settings                           | Editor              |
| `Ctrl+Shift+C` | Copy edits                                       | Editor, Grid, Loupe |
| `Ctrl+Shift+V` | Paste edits                                      | Editor, Grid, Loupe |
| `Ctrl+Shift+E` | Open Export                                      | Editor              |
| `Esc`          | Step out of the active tool, panel or fullscreen | Editor              |

## Sliders

| Keys                             | Action                                                       | Available in |
| -------------------------------- | ------------------------------------------------------------ | ------------ |
| `Shift+← / Shift+→`              | Move a focused slider ten steps                              | Editor       |
| `Double-click the label`         | Reset one slider                                             | Editor       |
| `Alt + drag`                     | Preview the mask or radius a slider drives while you drag it | Editor       |
| `Shift + click the reset button` | Reset every band or channel at once, on HSL and Curves       | Editor       |

## Geometry

| Keys                           | Action                                                            | Available in |
| ------------------------------ | ----------------------------------------------------------------- | ------------ |
| `Enter`                        | Apply the crop and transform and return to Develop                | Geometry     |
| `Shift+A`                      | Draw a line along the horizon to level the photo                  | Geometry     |
| `Shift + drag inside the crop` | Draw a straighten line without arming the tool                    | Geometry     |
| `O`                            | Cycle the crop guide: thirds, golden ratio, diagonals, grid, none | Geometry     |
| `Shift+X`                      | Swap the crop between landscape and portrait                      | Geometry     |
| `Alt + drag a crop handle`     | Resize the crop about its centre                                  | Geometry     |
| `Shift + drag a crop handle`   | Keep the current shape of a free crop while resizing              | Geometry     |
| `Ctrl + scroll over the crop`  | Scale the crop about its centre                                   | Geometry     |
| `Ctrl + drag inside the crop`  | Move the photo under a fixed crop                                 | Geometry     |

## Masks

| Keys                                                      | Action                                                        | Available in   |
| --------------------------------------------------------- | ------------------------------------------------------------- | -------------- |
| `Backspace` / `Del`                                       | Delete the selected shape, or undo the last polygon corner    | Masks          |
| `K`                                                       | Add a brush mask layer                                        | Editor         |
| `Shift+M`                                                 | Add a radial mask layer                                       | Editor         |
| `Shift+L`                                                 | Add a linear mask layer                                       | Editor         |
| `↑` / `↓` / `Shift+←` / `Shift+→` / `Shift+↑` / `Shift+↓` | Nudge the selected shape one pixel, ten with Shift            | Masks          |
| `Click the dot between two corners`                       | Add a polygon corner                                          | Masks          |
| `Double-click the corner`                                 | Remove a polygon corner                                       | Masks          |
| `Shift + drag a radial size handle`                       | Keep a radial round while resizing it                         | Masks          |
| `Shift + drag a linear end`                               | Snap a linear gradient to 45° steps                           | Masks          |
| `Shift + drag a polygon corner`                           | Move a polygon corner straight across or straight up and down | Masks          |
| `O`                                                       | Toggle the mask overlay                                       | Masks          |
| `Esc`                                                     | Cancel drawing, box select or the eyedropper                  | Masks          |
| `Enter`                                                   | Close the polygon you are drawing                             | Masks          |
| `[` / `]`                                                 | Smaller / larger brush                                        | Masks, Retouch |
| `{` / `}`                                                 | Softer / harder brush                                         | Masks, Retouch |

## Retouch

| Keys                  | Action                            | Available in |
| --------------------- | --------------------------------- | ------------ |
| `H`                   | Heal mode                         | Retouch      |
| `C`                   | Clone mode                        | Retouch      |
| `Backspace` / `Del`   | Delete the selected stroke        | Retouch      |
| `Esc`                 | Deselect the current stroke       | Retouch      |
| `Alt + click`         | Set the source point              | Retouch      |
| `Drag the green ring` | Move the selected stroke's source | Retouch      |
