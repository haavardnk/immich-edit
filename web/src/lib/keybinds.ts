export interface KeybindEntry {
  keys: string;
  description: string;
}

export type KeybindMode = 'grid' | 'loupe' | 'editor';

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
      { keys: 'E / Enter', description: 'Open editor' },
      { keys: 'Esc', description: 'Close loupe' }
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
      { keys: 'C', description: 'Open Geometry' },
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
  }
];
