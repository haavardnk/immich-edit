import { isMac } from '$lib/utils/platform';

export interface Chord {
  key: string;
  code: boolean;
  mod: boolean;
  shift: boolean;
  alt: boolean;
}

export function parseChord(spec: string): Chord {
  if (spec.length === 1) return { key: spec, code: false, mod: false, shift: false, alt: false };
  const parts = spec.split('+');
  const key = parts.pop() ?? '';
  return {
    key: key.length === 1 ? key.toLowerCase() : key,
    code: /^Digit\d$/.test(key),
    mod: parts.includes('Mod'),
    shift: parts.includes('Shift'),
    alt: parts.includes('Alt')
  };
}

function normalizeKey(key: string): string {
  if (key === ' ') return 'Space';
  return key.length === 1 ? key.toLowerCase() : key;
}

export function chordMatches(e: KeyboardEvent, chord: Chord): boolean {
  if (chord.mod !== (e.metaKey || e.ctrlKey)) return false;
  if (chord.alt !== e.altKey) return false;
  if ((chord.code ? e.code : normalizeKey(e.key)) !== chord.key) return false;
  if (chord.shift) return e.shiftKey;
  const shiftSensitive = chord.key.length > 1 || (chord.key >= 'a' && chord.key <= 'z');
  return shiftSensitive ? !e.shiftKey : true;
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
  const digit = /^Digit(\d)$/.exec(key);
  if (digit?.[1]) return digit[1];
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
