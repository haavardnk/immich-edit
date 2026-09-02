import { isMac } from './platform';

export type KeybindContext =
  'global' | 'grid' | 'loupe' | 'compare' | 'survey' | 'editor' | 'masks' | 'retouch';

export interface Keybind {
  readonly id: string;
  readonly keys: readonly string[];
  readonly contexts: readonly KeybindContext[];
  readonly group: string;
  readonly label: string;
  readonly display?: string;
}

export const KEYBINDS = [
  {
    id: 'help',
    keys: ['?', 'Shift+/'],
    contexts: ['global'],
    group: 'General',
    label: 'Show keyboard shortcuts',
    display: '?'
  },
  {
    id: 'backToGrid',
    keys: ['g'],
    contexts: ['loupe', 'compare', 'survey', 'editor'],
    group: 'General',
    label: 'Back to the grid'
  },

  {
    id: 'rate',
    keys: ['0', '1', '2', '3', '4', '5'],
    contexts: ['grid', 'loupe', 'compare', 'survey', 'editor'],
    group: 'Culling',
    label: 'Set, toggle or clear the rating',
    display: '0 – 5'
  },
  {
    id: 'favorite',
    keys: ['p', 'f'],
    contexts: ['grid', 'loupe', 'compare', 'survey', 'editor'],
    group: 'Culling',
    label: 'Toggle favorite'
  },
  {
    id: 'reject',
    keys: ['x'],
    contexts: ['grid', 'loupe', 'compare', 'survey', 'editor'],
    group: 'Culling',
    label: 'Toggle reject'
  },
  {
    id: 'unflag',
    keys: ['u'],
    contexts: ['grid', 'loupe', 'compare', 'survey', 'editor'],
    group: 'Culling',
    label: 'Clear favorite and reject'
  },

  {
    id: 'gridMove',
    keys: ['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown'],
    contexts: ['grid'],
    group: 'Grid',
    label: 'Move the active photo'
  },
  {
    id: 'gridEdge',
    keys: ['Home', 'End'],
    contexts: ['grid'],
    group: 'Grid',
    label: 'First / last photo'
  },
  {
    id: 'gridPage',
    keys: ['PageUp', 'PageDown'],
    contexts: ['grid'],
    group: 'Grid',
    label: 'Jump a page'
  },
  {
    id: 'gridSize',
    keys: ['-', '_', '=', '+'],
    contexts: ['grid'],
    group: 'Grid',
    label: 'Thumbnail size',
    display: '− / +'
  },
  {
    id: 'gridSelectAll',
    keys: ['Mod+a'],
    contexts: ['grid'],
    group: 'Grid',
    label: 'Select every loaded photo'
  },
  {
    id: 'gridClearSelection',
    keys: ['Escape'],
    contexts: ['grid'],
    group: 'Grid',
    label: 'Clear the selection'
  },
  {
    id: 'openLoupe',
    keys: ['e', 'Space'],
    contexts: ['grid'],
    group: 'Grid',
    label: 'Open the loupe'
  },
  {
    id: 'openEditor',
    keys: ['d', 'Enter'],
    contexts: ['grid', 'loupe'],
    group: 'Grid',
    label: 'Open the editor'
  },
  {
    id: 'enterCompare',
    keys: ['c'],
    contexts: ['grid', 'loupe'],
    group: 'Grid',
    label: 'Compare the selected photos'
  },
  {
    id: 'enterSurvey',
    keys: ['n'],
    contexts: ['grid', 'loupe'],
    group: 'Grid',
    label: 'Survey the selected photos'
  },

  {
    id: 'loupeNav',
    keys: ['ArrowLeft', 'ArrowRight'],
    contexts: ['loupe'],
    group: 'Loupe',
    label: 'Previous / next photo'
  },
  {
    id: 'zoomToggle',
    keys: ['z', 'Space'],
    contexts: ['loupe', 'compare', 'survey', 'editor'],
    group: 'Loupe',
    label: 'Toggle zoom'
  },
  {
    id: 'toggleInfo',
    keys: ['i'],
    contexts: ['loupe', 'editor'],
    group: 'Loupe',
    label: 'Toggle the info panel'
  },
  {
    id: 'toggleTags',
    keys: ['t'],
    contexts: ['loupe', 'editor'],
    group: 'Loupe',
    label: 'Toggle the tags panel'
  },
  {
    id: 'clipWarn',
    keys: ['j'],
    contexts: ['loupe', 'compare', 'survey', 'editor'],
    group: 'Loupe',
    label: 'Toggle the clipping indicators'
  },
  {
    id: 'loupeClose',
    keys: ['Escape'],
    contexts: ['loupe'],
    group: 'Loupe',
    label: 'Close the loupe'
  },

  {
    id: 'compareFocus',
    keys: ['ArrowLeft', 'ArrowRight'],
    contexts: ['compare'],
    group: 'Compare',
    label: 'Move focus between panes'
  },
  {
    id: 'paneFocusCycle',
    keys: ['Tab', 'Shift+Tab'],
    contexts: ['compare', 'survey'],
    group: 'Compare',
    label: 'Cycle focus between panes'
  },
  {
    id: 'paneSwap',
    keys: ['Shift+ArrowLeft', 'Shift+ArrowRight'],
    contexts: ['compare', 'survey'],
    group: 'Compare',
    label: 'Swap the focused pane for another photo'
  },
  {
    id: 'paneSync',
    keys: ['y'],
    contexts: ['compare', 'survey'],
    group: 'Compare',
    label: 'Toggle synced zoom and pan'
  },
  {
    id: 'paneOpenEditor',
    keys: ['d'],
    contexts: ['compare', 'survey'],
    group: 'Compare',
    label: 'Open the focused photo in the editor'
  },
  {
    id: 'paneDrop',
    keys: ['Backspace', 'Delete'],
    contexts: ['compare', 'survey'],
    group: 'Compare',
    label: 'Drop the focused photo'
  },
  {
    id: 'panePromote',
    keys: ['Enter'],
    contexts: ['compare'],
    group: 'Compare',
    label: 'Promote the focused pane to the left'
  },
  {
    id: 'compareExit',
    keys: ['e', 'Escape'],
    contexts: ['compare'],
    group: 'Compare',
    label: 'Back to the loupe on the focused photo'
  },

  {
    id: 'surveyFocus',
    keys: ['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown'],
    contexts: ['survey'],
    group: 'Survey',
    label: 'Move focus between panes'
  },
  {
    id: 'surveyKeep',
    keys: ['Enter'],
    contexts: ['survey'],
    group: 'Survey',
    label: 'Keep only the focused photo'
  },
  {
    id: 'surveyExit',
    keys: ['e', 'Escape'],
    contexts: ['survey'],
    group: 'Survey',
    label: 'Back to the loupe, selecting the survivors if you dropped any'
  },

  {
    id: 'editorNav',
    keys: ['ArrowLeft', 'ArrowRight'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Previous / next photo'
  },
  {
    id: 'undo',
    keys: ['Mod+z'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Undo'
  },
  {
    id: 'redo',
    keys: ['Mod+Shift+z'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Redo'
  },
  {
    id: 'openDevelop',
    keys: ['d'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Open Develop'
  },
  {
    id: 'openGeometry',
    keys: ['r'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Open Geometry'
  },
  {
    id: 'openRetouch',
    keys: ['q'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Open Retouch'
  },
  {
    id: 'openMasks',
    keys: ['m'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Open Masks'
  },
  {
    id: 'createVirtualCopy',
    keys: ["Mod+'"],
    contexts: ['grid', 'loupe', 'editor'],
    group: 'Editor',
    label: 'Create a virtual copy'
  },
  {
    id: 'perspective',
    keys: ['Shift+p'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Toggle the perspective corner handles'
  },
  {
    id: 'beforeAfter',
    keys: ['y'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Toggle the before / after split'
  },
  {
    id: 'holdOriginal',
    keys: ['\\'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Hold to view the original',
    display: '\\ (hold)'
  },
  {
    id: 'togglePanels',
    keys: ['Tab'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Hide or show the side panels'
  },
  {
    id: 'toggleChrome',
    keys: ['Shift+Tab'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Hide or show every panel'
  },
  {
    id: 'fullscreen',
    keys: ['Shift+f'],
    contexts: ['loupe', 'compare', 'survey', 'editor'],
    group: 'Editor',
    label: 'Toggle fullscreen'
  },
  {
    id: 'resetEdits',
    keys: ['Mod+Shift+r'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Reset every edit'
  },
  {
    id: 'copyEdits',
    keys: ['Mod+Shift+c'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Copy edits'
  },
  {
    id: 'pasteEdits',
    keys: ['Mod+Shift+v'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Paste edits'
  },
  {
    id: 'openExport',
    keys: ['Mod+Shift+e'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Open Export'
  },
  {
    id: 'editorEscape',
    keys: ['Escape'],
    contexts: ['editor'],
    group: 'Editor',
    label: 'Step out of the active tool, panel or fullscreen'
  },

  {
    id: 'maskDelete',
    keys: ['Backspace', 'Delete'],
    contexts: ['masks'],
    group: 'Masks',
    label: 'Delete the selected shape, or undo the last polygon corner'
  },
  {
    id: 'maskOverlay',
    keys: ['o'],
    contexts: ['masks'],
    group: 'Masks',
    label: 'Toggle the mask overlay'
  },
  {
    id: 'maskCancelDraw',
    keys: ['Escape'],
    contexts: ['masks'],
    group: 'Masks',
    label: 'Cancel drawing, box select or the eyedropper'
  },
  {
    id: 'maskClosePolygon',
    keys: ['Enter'],
    contexts: ['masks'],
    group: 'Masks',
    label: 'Close the polygon you are drawing'
  },
  {
    id: 'brushSize',
    keys: ['[', ']'],
    contexts: ['masks', 'retouch'],
    group: 'Masks',
    label: 'Smaller / larger brush'
  },
  {
    id: 'brushHardness',
    keys: ['{', '}'],
    contexts: ['masks', 'retouch'],
    group: 'Masks',
    label: 'Softer / harder brush'
  },

  {
    id: 'retouchHeal',
    keys: ['h'],
    contexts: ['retouch'],
    group: 'Retouch',
    label: 'Heal mode'
  },
  {
    id: 'retouchClone',
    keys: ['c'],
    contexts: ['retouch'],
    group: 'Retouch',
    label: 'Clone mode'
  },
  {
    id: 'retouchDelete',
    keys: ['Backspace', 'Delete'],
    contexts: ['retouch'],
    group: 'Retouch',
    label: 'Delete the selected stroke'
  },
  {
    id: 'retouchDeselect',
    keys: ['Escape'],
    contexts: ['retouch'],
    group: 'Retouch',
    label: 'Deselect the current stroke'
  }
] as const satisfies readonly Keybind[];

export type KeybindId = (typeof KEYBINDS)[number]['id'];

interface Chord {
  key: string;
  mod: boolean;
  shift: boolean;
  alt: boolean;
}

function parseChord(spec: string): Chord {
  if (spec.length === 1) return { key: spec, mod: false, shift: false, alt: false };
  const parts = spec.split('+');
  const key = parts.pop() ?? '';
  return {
    key: key.length === 1 ? key.toLowerCase() : key,
    mod: parts.includes('Mod'),
    shift: parts.includes('Shift'),
    alt: parts.includes('Alt')
  };
}

const CHORDS = new Map<string, Chord[]>(KEYBINDS.map((b) => [b.id, b.keys.map(parseChord)]));

function normalizeKey(key: string): string {
  if (key === ' ') return 'Space';
  return key.length === 1 ? key.toLowerCase() : key;
}

function chordMatches(e: KeyboardEvent, chord: Chord): boolean {
  if (chord.mod !== (e.metaKey || e.ctrlKey)) return false;
  if (chord.alt !== e.altKey) return false;
  if (normalizeKey(e.key) !== chord.key) return false;
  if (chord.shift) return e.shiftKey;
  const shiftSensitive = chord.key.length > 1 || (chord.key >= 'a' && chord.key <= 'z');
  return shiftSensitive ? !e.shiftKey : true;
}

export function isKeybind(e: KeyboardEvent, id: KeybindId): boolean {
  return (CHORDS.get(id) ?? []).some((chord) => chordMatches(e, chord));
}

export function isTypingTarget(e: KeyboardEvent): boolean {
  const el = e.target as HTMLElement | null;
  if (!el) return false;
  const tag = el.tagName;
  if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return true;
  return el.isContentEditable;
}

export function isRadioGroupTarget(e: KeyboardEvent): boolean {
  const el = e.target as HTMLElement | null;
  return !!el?.closest('[role="radiogroup"]');
}

export function matchKeybind(
  e: KeyboardEvent,
  contexts: readonly KeybindContext[]
): KeybindId | null {
  for (const bind of KEYBINDS) {
    if (!bind.contexts.some((c) => contexts.includes(c))) continue;
    if (isKeybind(e, bind.id)) return bind.id;
  }
  return null;
}

const MAC_KEYS: Record<string, string> = {
  Mod: '⌘',
  Shift: '⇧',
  Alt: '⌥',
  Escape: 'Esc',
  Enter: 'Return',
  Backspace: '⌫',
  Delete: '⌦',
  Tab: 'Tab'
};

const PC_KEYS: Record<string, string> = {
  Mod: 'Ctrl',
  Shift: 'Shift',
  Alt: 'Alt',
  Escape: 'Esc',
  Enter: 'Enter',
  Backspace: 'Backspace',
  Delete: 'Del',
  Tab: 'Tab'
};

const SHARED_KEYS: Record<string, string> = {
  ArrowLeft: '←',
  ArrowRight: '→',
  ArrowUp: '↑',
  ArrowDown: '↓',
  PageUp: 'PgUp',
  PageDown: 'PgDn',
  Space: 'Space',
  Home: 'Home',
  End: 'End'
};

export function keyLabel(key: string, mac: boolean = isMac): string {
  const shared = SHARED_KEYS[key];
  if (shared) return shared;
  const named = (mac ? MAC_KEYS : PC_KEYS)[key];
  if (named) return named;
  return key.length === 1 ? key.toUpperCase() : key;
}

export function formatChord(spec: string, mac: boolean = isMac): string {
  const chord = parseChord(spec);
  const parts: string[] = [];
  if (chord.mod) parts.push(keyLabel('Mod', mac));
  if (chord.alt) parts.push(keyLabel('Alt', mac));
  if (chord.shift) parts.push(keyLabel('Shift', mac));
  parts.push(keyLabel(chord.key, mac));
  return parts.join(mac ? '' : '+');
}

export function keysFor(id: KeybindId, mac: boolean = isMac): string {
  const bind = (KEYBINDS as readonly Keybind[]).find((b) => b.id === id);
  if (!bind) return '';
  if (bind.display) return bind.display;
  return bind.keys.map((spec) => formatChord(spec, mac)).join(' / ');
}

export function hint(label: string, id: KeybindId, mac: boolean = isMac): string {
  return `${label} (${keysFor(id, mac)})`;
}
