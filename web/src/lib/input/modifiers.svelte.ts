export type ModifierKeys = {
  shiftKey: boolean;
  altKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
};

type ModifierKey = 'shift' | 'alt' | 'mod';

const ROLES = {
  constrain: 'shift',
  straighten: 'shift',
  fromCentre: 'alt',
  pan: 'mod',
  zoom: 'mod',
  coarse: 'shift',
  fine: 'alt',
  exclude: 'shift',
  sample: 'alt',
  extend: 'shift',
  toggle: 'mod',
  solo: 'alt',
  preview: 'alt',
  resetAll: 'shift',
  resize: 'alt',
  hardness: 'shift',
  feather: 'shift',
  erase: 'alt',
  lineBreak: 'shift',
  hueAxis: 'alt',
  lightnessAxis: 'shift'
} as const satisfies Record<string, ModifierKey>;

export type ModifierRole = keyof typeof ROLES;

const KEYS: readonly ModifierKey[] = ['shift', 'alt', 'mod'];

function down(key: ModifierKey, e: ModifierKeys): boolean {
  if (key === 'shift') return e.shiftKey;
  if (key === 'alt') return e.altKey;
  return e.ctrlKey || e.metaKey;
}

class ModifierLatches {
  roles = $state<ModifierRole[]>([]);

  has(role: ModifierRole): boolean {
    return this.roles.includes(role);
  }

  set(role: ModifierRole, on: boolean): void {
    if (on === this.has(role)) return;
    this.roles = on ? [...this.roles, role] : this.roles.filter((r) => r !== role);
  }
}

const latches = new ModifierLatches();

export function latch(role: ModifierRole, on: boolean): void {
  latches.set(role, on);
}

export function latched(role: ModifierRole): boolean {
  return latches.has(role);
}

export function clear(): void {
  latches.roles = [];
}

export function keyHeld(role: ModifierRole, e: ModifierKeys): boolean {
  return down(ROLES[role], e);
}

export function held(role: ModifierRole, e: ModifierKeys): boolean {
  return latched(role) || keyHeld(role, e);
}

export function heldAlone(role: ModifierRole, e: ModifierKeys): boolean {
  const own = ROLES[role];
  return held(role, e) && !KEYS.some((key) => key !== own && down(key, e));
}

export function unmodified(e: ModifierKeys): boolean {
  return !KEYS.some((key) => down(key, e));
}

export function plainClick(e: ModifierKeys & { button: number }): boolean {
  return e.button === 0 && unmodified(e);
}
