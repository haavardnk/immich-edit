export interface KeybindEntry {
  keys: string;
  description: string;
}

export const KEYBINDS: KeybindEntry[] = [
  { keys: '← / → · J / K', description: 'Previous / next asset' },
  { keys: 'Space / Z', description: 'Toggle zoom (fit ↔ 200%)' },
  { keys: 'I', description: 'Toggle EXIF info' },
  { keys: 'T', description: 'Toggle tags' },
  { keys: 'R', description: 'Reset edits' },
  { keys: 'B / \\ (hold)', description: 'Before / after — toggle or hold for original' },
  { keys: '1 – 5', description: 'Set / clear star rating' },
  { keys: '0', description: 'Clear star rating' },
  { keys: 'F', description: 'Toggle favorite' },
  { keys: '⇧F', description: 'Toggle fullscreen' },
  { keys: '⌘Z / ⌘⇧Z', description: 'Undo / redo' },
  { keys: 'Esc', description: 'Exit crop / brush / popover / fullscreen' },
  { keys: '?', description: 'Toggle this help' }
];
