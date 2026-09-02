---
layout: default
title: Keyboard shortcuts
nav_order: 4
permalink: /shortcuts/
---

# Keyboard shortcuts

Press `?` anywhere in immich-edit to open the searchable shortcut list. On macOS, `Mod` means
Command. On Windows and Linux, it means Ctrl.

Shortcuts do not run while focus is in a text field or other typing control.

## General and culling

| Keys       | Action                       | Available in                         |
| ---------- | ---------------------------- | ------------------------------------ |
| `?`        | Show keyboard shortcuts      | Anywhere                             |
| `G`        | Return to the grid           | Loupe, compare, survey, editor       |
| `0`-`5`    | Set, toggle, or clear rating | Grid, loupe, compare, survey, editor |
| `P` or `F` | Toggle favorite              | Grid, loupe, compare, survey, editor |
| `X`        | Toggle reject                | Grid, loupe, compare, survey, editor |
| `U`        | Clear favorite and reject    | Grid, loupe, compare, survey, editor |

## Grid

| Keys                     | Action                          | Available in   |
| ------------------------ | ------------------------------- | -------------- |
| Arrow keys               | Move the active photo           | Grid           |
| `Home` or `End`          | Move to the first or last photo | Grid           |
| `Page Up` or `Page Down` | Jump one page                   | Grid           |
| `-` or `+`               | Change thumbnail size           | Grid           |
| `Mod+A`                  | Select every loaded photo       | Grid           |
| `Escape`                 | Clear the selection             | Grid           |
| `E` or `Space`           | Open the loupe                  | Grid           |
| `D` or `Enter`           | Open the editor                 | Grid and loupe |
| `C`                      | Compare selected photos         | Grid and loupe |
| `N`                      | Survey selected photos          | Grid and loupe |

## Loupe

| Keys           | Action                               | Available in                   |
| -------------- | ------------------------------------ | ------------------------------ |
| Left or Right  | Open the previous or next photo      | Loupe                          |
| `Z` or `Space` | Toggle zoom                          | Loupe, compare, survey, editor |
| `I`            | Toggle photo information             | Loupe and editor               |
| `T`            | Toggle tags                          | Loupe and editor               |
| `J`            | Toggle clipping indicators           | Loupe, compare, survey, editor |
| `Shift+F`      | Toggle fullscreen                    | Loupe, compare, survey, editor |
| `Escape`       | Leave fullscreen or close the loupe  | Loupe                          |

## Compare and survey

| Keys                          | Action                                          | Available in       |
| ----------------------------- | ----------------------------------------------- | ------------------ |
| Left or Right                 | Move focus between photos                       | Compare            |
| Arrow keys                    | Move focus between photos                       | Survey             |
| `Tab` or `Shift+Tab`          | Cycle focus between photos                      | Compare and survey |
| `Shift+Left` or `Shift+Right` | Replace the focused photo                       | Compare and survey |
| `Y`                           | Toggle synchronized zoom and pan                | Compare and survey |
| `D`                           | Open the focused photo in the editor            | Compare and survey |
| `Backspace` or `Delete`       | Drop the focused photo                          | Compare and survey |
| `Enter`                       | Promote the focused photo to the left           | Compare            |
| `Enter`                       | Keep only the focused photo                     | Survey             |
| `E` or `Escape`               | Return to the loupe on the focused photo        | Compare            |
| `E` or `Escape`               | Return to the loupe and select surviving photos | Survey             |

## Editor

| Keys          | Action                                           |
| ------------- | ------------------------------------------------ |
| `D`           | Open **Develop**                                 |
| Left or Right | Open the previous or next photo                  |
| `Mod+Z`       | Undo                                             |
| `Mod+Shift+Z` | Redo                                             |
| `R`           | Open **Geometry**                                |
| `Q`           | Open **Retouch**                                 |
| `M`           | Open **Masks**                                   |
| `Mod+'`       | Create a virtual copy                            |
| `Shift+P`     | Toggle perspective corner handles                |
| `Y`           | Toggle the before-and-after split                |
| `\` (hold)    | Show the original                                |
| `Tab`         | Hide or show side panels                         |
| `Shift+Tab`   | Hide or show every panel                         |
| `Shift+F`     | Toggle fullscreen                                |
| `Mod+Shift+R` | Reset every edit                                 |
| `Mod+Shift+C` | Copy edits                                       |
| `Mod+Shift+V` | Paste edits                                      |
| `Mod+Shift+E` | Open **Export**                                  |
| `Escape`      | Leave the active tool, panel, or fullscreen mode |

## Masks and retouch

| Keys                    | Action                                                    | Available in      |
| ----------------------- | --------------------------------------------------------- | ----------------- |
| `Backspace` or `Delete` | Delete the selected shape or undo the last polygon corner | Masks             |
| `O`                     | Toggle the mask overlay                                   | Masks             |
| `Escape`                | Cancel drawing, box selection, or the eyedropper          | Masks             |
| `Enter`                 | Close the polygon being drawn                             | Masks             |
| `[` or `]`              | Decrease or increase brush size                           | Masks and retouch |
| `{` or `}`              | Decrease or increase brush hardness                       | Masks and retouch |
| `H`                     | Select Heal mode                                          | Retouch           |
| `C`                     | Select Clone mode                                         | Retouch           |
| `Backspace` or `Delete` | Delete the selected stroke                                | Retouch           |
| `Escape`                | Deselect the current stroke                               | Retouch           |
