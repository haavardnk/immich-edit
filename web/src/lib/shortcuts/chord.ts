import { isMac } from '#lib/utils/platform.ts';

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

function isLetter(key: string): boolean {
  return /^[a-z]$/.test(key);
}

function keyMatches(e: KeyboardEvent, chord: Chord): boolean {
  if (chord.code) return e.code === chord.key;
  if (normalizeKey(e.key) === chord.key) return true;
  return chord.alt && isLetter(chord.key) && e.code === `Key${chord.key.toUpperCase()}`;
}

export function chordMatches(e: KeyboardEvent, chord: Chord): boolean {
  const layoutSymbol =
    !chord.alt && !chord.code && chord.key.length === 1 && !/[a-z0-9]/.test(chord.key);
  const altGraph = layoutSymbol && e.getModifierState('AltGraph');
  if (chord.mod !== (e.metaKey || (e.ctrlKey && !altGraph))) return false;
  if (!layoutSymbol && chord.alt !== e.altKey) return false;
  if (!keyMatches(e, chord)) return false;
  if (chord.shift) return e.shiftKey;
  const shiftSensitive = chord.key.length > 1 || (chord.key >= 'a' && chord.key <= 'z');
  return shiftSensitive ? !e.shiftKey : true;
}

const UNRECORDABLE = new Set([
  'Shift',
  'Control',
  'Alt',
  'Meta',
  'AltGraph',
  'CapsLock',
  'Fn',
  'FnLock',
  'Hyper',
  'Super',
  'OS',
  'NumLock',
  'ScrollLock',
  'Symbol',
  'SymbolLock',
  'Escape',
  'Dead',
  'Process',
  'Unidentified'
]);

export function isModifierOnly(e: KeyboardEvent): boolean {
  return UNRECORDABLE.has(e.key) && e.key !== 'Escape';
}

function specKey(e: KeyboardEvent): string | null {
  const typed = e.key.length === 1 ? e.key.toLowerCase() : e.key;
  if (isLetter(typed)) return typed;
  const letter = /^Key([A-Z])$/.exec(e.code)?.[1];
  if (letter && (e.altKey || e.metaKey || e.ctrlKey)) return letter.toLowerCase();
  const digit = /^Digit(\d)$/.exec(e.code)?.[1];
  if (digit && (e.shiftKey || e.altKey)) return `Digit${digit}`;
  if (e.key === ' ') return 'Space';
  if (/^[!-~]$/.test(e.key)) return e.key;
  if (/^[A-Z][A-Za-z0-9]*$/.test(e.key)) return e.key;
  return null;
}

export function chordSpec(e: KeyboardEvent): string | null {
  if (UNRECORDABLE.has(e.key)) return null;
  const key = specKey(e);
  if (!key) return null;
  const symbol = key.length === 1 && !/[a-z0-9]/.test(key);
  const altGraph = symbol && e.getModifierState('AltGraph');
  const parts: string[] = [];
  if (e.metaKey || (e.ctrlKey && !altGraph)) parts.push('Mod');
  if (e.altKey && !symbol) parts.push('Alt');
  if (e.shiftKey && !symbol) parts.push('Shift');
  parts.push(key);
  return parts.join('+');
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
