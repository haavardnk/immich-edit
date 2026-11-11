import { parseChord } from './chord';
import { sharedContext } from './stacks';
import { KEYBINDS, type Keybind, type KeybindContext } from './table';

export type ShortcutMap = Readonly<Record<string, readonly string[]>>;

export interface Conflict {
  bind: Keybind;
  context: KeybindContext;
}

const BINDS = KEYBINDS as readonly Keybind[];
const BY_ID = new Map(BINDS.map((bind) => [bind.id, bind]));

const BROWSER_OWNED = new Set([
  ...'0123456789'.split('').map((digit) => `Mod+${digit}`),
  'Mod+t',
  'Mod+w',
  'Mod+n',
  'Mod+q',
  'Mod+r',
  'Mod+l',
  'Mod+=',
  'Mod+-',
  'Mod+Shift+t',
  'Mod+Shift+n',
  'Mod+Shift+w'
]);

export function bindOf(id: string): Keybind | undefined {
  return BY_ID.get(id);
}

export function isCustomisable(bind: Keybind): boolean {
  return bind.fixed !== true && bind.keys.length > 0;
}

export function isCustomised(id: string, custom: ShortcutMap): boolean {
  const bind = BY_ID.get(id);
  return bind !== undefined && isCustomisable(bind) && Object.hasOwn(custom, id);
}

export function keysOf(id: string, custom: ShortcutMap): readonly string[] {
  const bind = BY_ID.get(id);
  if (!bind) return [];
  return isCustomised(id, custom) ? (custom[id] ?? bind.keys) : bind.keys;
}

export function customCount(custom: ShortcutMap): number {
  return Object.keys(custom).filter((id) => isCustomised(id, custom)).length;
}

export function browserOwned(spec: string): boolean {
  return BROWSER_OWNED.has(spec);
}

export function sameChord(a: string, b: string): boolean {
  const left = parseChord(a);
  const right = parseChord(b);
  return (
    left.key === right.key &&
    left.code === right.code &&
    left.mod === right.mod &&
    left.alt === right.alt &&
    left.shift === right.shift
  );
}

export function conflictOf(id: string, spec: string, custom: ShortcutMap): Conflict | null {
  const bind = BY_ID.get(id);
  if (!bind) return null;
  for (const other of BINDS) {
    if (other.id === id) continue;
    if (!keysOf(other.id, custom).some((key) => sameChord(key, spec))) continue;
    const context = sharedContext(bind, other);
    if (context) return { bind: other, context };
  }
  return null;
}

function sameKeys(a: readonly string[], b: readonly string[]): boolean {
  return a.length === b.length && a.every((key, index) => key === b[index]);
}

function normalised(next: Record<string, readonly string[]>): ShortcutMap {
  const result: Record<string, readonly string[]> = {};
  for (const [id, keys] of Object.entries(next)) {
    const bind = BY_ID.get(id);
    if (bind && isCustomisable(bind) && sameKeys(keys, bind.keys)) continue;
    result[id] = keys;
  }
  return result;
}

export function bindKey(custom: ShortcutMap, id: string, spec: string): ShortcutMap {
  return normalised({ ...custom, [id]: [spec] });
}

export function swapKey(custom: ShortcutMap, id: string, spec: string, other: string): ShortcutMap {
  const given = keysOf(id, custom);
  const swapped = keysOf(other, custom).flatMap((key) => (sameChord(key, spec) ? given : [key]));
  return normalised({ ...custom, [id]: [spec], [other]: [...new Set(swapped)] });
}

export function resetKey(custom: ShortcutMap, id: string): ShortcutMap {
  return Object.fromEntries(Object.entries(custom).filter(([key]) => key !== id));
}
