export interface KeybindEntry {
  keys: string;
  description: string;
}

export type KeybindMode = 'grid' | 'loupe' | 'compare' | 'survey' | 'editor';

export interface KeybindGroup {
  title: string;
  mode: KeybindMode;
  binds: KeybindEntry[];
}

export const KEYBIND_GROUPS: KeybindGroup[] = [
  {
    title: 'Grid',
    mode: 'grid',
    binds: [
      { keys: '← / → / ↑ / ↓', description: 'Move active photo' },
      { keys: 'Home / End', description: 'First / last photo' },
      { keys: 'PgUp / PgDn', description: 'Jump a page' },
      { keys: '1 – 5 / 0', description: 'Set / toggle / clear rating' },
      { keys: 'F', description: 'Toggle favorite' },
      { keys: 'X', description: 'Toggle reject' },
      { keys: '− / +', description: 'Thumbnail size' },
      { keys: 'Enter', description: 'Open editor' },
      { keys: 'Space', description: 'Open quick-review loupe' }
    ]
  },
  {
    title: 'Loupe',
    mode: 'loupe',
    binds: [
      { keys: '← / → · J / K', description: 'Previous / next photo' },
      { keys: '1 – 5 / 0', description: 'Set / toggle / clear rating' },
      { keys: 'F', description: 'Toggle favorite' },
      { keys: 'X', description: 'Toggle reject' },
      { keys: 'Z / Space', description: 'Toggle zoom' },
      { keys: 'I', description: 'Toggle info' },
      { keys: 'T', description: 'Toggle tags' },
      { keys: 'C', description: 'Compare two photos' },
      { keys: 'N', description: 'Survey several photos' },
      { keys: 'E / Enter', description: 'Open editor' },
      { keys: 'Esc', description: 'Close loupe' }
    ]
  },
  {
    title: 'Compare',
    mode: 'compare',
    binds: [
      { keys: '← / → · Tab', description: 'Move focus between panes' },
      { keys: 'J / K', description: 'Swap the focused pane for another photo' },
      { keys: 'Enter', description: 'Promote the focused pane to the left' },
      { keys: 'Z / Space', description: 'Toggle zoom' },
      { keys: 'Y', description: 'Toggle synced zoom and pan' },
      { keys: 'Alt-drag', description: 'Pan only the pane under the pointer' },
      { keys: '1 – 5 / 0 · F · X', description: 'Rate, favorite or reject the focused pane' },
      { keys: 'Esc', description: 'Back to a single photo' }
    ]
  },
  {
    title: 'Survey',
    mode: 'survey',
    binds: [
      { keys: '← / → / ↑ / ↓ · Tab', description: 'Move focus between panes' },
      { keys: '⌫ / Del', description: 'Drop the focused photo from the survey' },
      { keys: 'Enter', description: 'Keep only the focused photo' },
      { keys: 'Z / Space', description: 'Toggle zoom' },
      { keys: 'Y', description: 'Toggle synced zoom and pan' },
      { keys: '1 – 5 / 0 · F · X', description: 'Rate, favorite or reject the focused pane' },
      { keys: 'Esc', description: 'Exit and select the survivors' }
    ]
  },
  {
    title: 'Editor',
    mode: 'editor',
    binds: [
      { keys: '← / → · J / K', description: 'Previous / next asset' },
      { keys: 'Space / Z', description: 'Toggle zoom (fit ↔ 200%)' },
      { keys: 'I', description: 'Toggle EXIF info' },
      { keys: 'T', description: 'Toggle tags' },
      { keys: 'G', description: 'Open Geometry' },
      { keys: 'P', description: 'Toggle perspective corner handles' },
      { keys: 'V', description: 'Open Retouch' },
      { keys: 'R', description: 'Reset edits' },
      { keys: 'B / \\ (hold)', description: 'Before / after — toggle or hold for original' },
      { keys: '1 – 5 / 0', description: 'Set / toggle / clear rating' },
      { keys: 'F', description: 'Toggle favorite' },
      { keys: 'X', description: 'Toggle reject' },
      { keys: '⇧F', description: 'Toggle fullscreen' },
      { keys: '⌘Z / ⌘⇧Z', description: 'Undo / redo' },
      { keys: 'Esc', description: 'Close Geometry / brush / popover / fullscreen' },
      { keys: '?', description: 'Toggle this help' }
    ]
  },
  {
    title: 'Masks',
    mode: 'editor',
    binds: [
      { keys: '⌫ / Del', description: 'Delete the selected shape' },
      { keys: 'Double-click', description: 'Rename a mask · remove a polygon corner' },
      { keys: 'Esc', description: 'Cancel drawing, box select or the eyedropper' },
      { keys: '⇧Click', description: 'Exclude an area when clicking to select' },
      { keys: 'Right-click', description: 'Exclude an area · undo the last polygon corner' },
      { keys: 'Enter', description: 'Close the polygon you are drawing' }
    ]
  },
  {
    title: 'Retouch',
    mode: 'editor',
    binds: [
      { keys: 'H / C', description: 'Heal / clone mode' },
      { keys: 'Hold Alt + click', description: 'Set the source point' },
      { keys: '[ / ]', description: 'Smaller / larger brush' },
      { keys: '⌫ / Del', description: 'Delete the selected stroke' },
      { keys: 'Esc', description: 'Deselect the current stroke' }
    ]
  }
];
