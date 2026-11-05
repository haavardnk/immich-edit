import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { flushSync, mount, tick, unmount, type Component } from 'svelte';

export type Tristate = boolean | 'mixed';

export interface ControlRef {
  role: string;
  name: string;
}

export interface Control extends ControlRef {
  disabled?: true;
  checked?: Tristate;
  pressed?: Tristate;
  expanded?: boolean;
}

export interface LayoutOptions {
  exclude?: ControlRef[];
  open?: (root: HTMLElement) => void | Promise<void>;
}

export interface Layout<L extends string = string> {
  label: L;
  exclude: ControlRef[];
  open: (root: HTMLElement) => void | Promise<void>;
  render: (target: HTMLElement) => () => Promise<void>;
}

const WIDGET_ROLES = new Set([
  'button',
  'checkbox',
  'combobox',
  'link',
  'listbox',
  'menuitem',
  'menuitemcheckbox',
  'menuitemradio',
  'option',
  'radio',
  'searchbox',
  'slider',
  'spinbutton',
  'switch',
  'tab',
  'textbox'
]);

const CANDIDATES = 'button, a[href], input, select, textarea, [role]';

const INPUT_ROLES: Record<string, string> = {
  button: 'button',
  checkbox: 'checkbox',
  image: 'button',
  number: 'spinbutton',
  radio: 'radio',
  range: 'slider',
  reset: 'button',
  search: 'searchbox',
  submit: 'button'
};

function implicitRole(el: Element): string | null {
  if (el instanceof HTMLButtonElement) return 'button';
  if (el instanceof HTMLAnchorElement) return el.hasAttribute('href') ? 'link' : null;
  if (el instanceof HTMLTextAreaElement) return 'textbox';
  if (el instanceof HTMLSelectElement) return el.multiple || el.size > 1 ? 'listbox' : 'combobox';
  if (el instanceof HTMLInputElement) {
    if (el.type === 'hidden') return null;
    return INPUT_ROLES[el.type] ?? 'textbox';
  }
  return null;
}

function roleOf(el: Element): string | null {
  const explicit = el.getAttribute('role')?.trim().split(/\s+/)[0];
  return explicit || implicitRole(el);
}

function squash(text: string | null | undefined): string {
  return (text ?? '').replace(/\s+/g, ' ').trim();
}

function labelText(el: Element): string {
  const labels =
    el instanceof HTMLInputElement ||
    el instanceof HTMLSelectElement ||
    el instanceof HTMLTextAreaElement
      ? el.labels
      : null;
  return squash([...(labels ?? [])].map((label) => label.textContent).join(' '));
}

function nameOf(el: Element): string {
  const labelledBy = el.getAttribute('aria-labelledby');
  if (labelledBy) {
    const text = squash(
      labelledBy
        .split(/\s+/)
        .map((id) => document.getElementById(id)?.textContent ?? '')
        .join(' ')
    );
    if (text) return text;
  }
  const ariaLabel = squash(el.getAttribute('aria-label'));
  if (ariaLabel) return ariaLabel;
  const label = labelText(el);
  if (label) return label;
  if (!(el instanceof HTMLInputElement) && !(el instanceof HTMLSelectElement)) {
    const text = squash(el.textContent);
    if (text) return text;
  }
  return squash(el.getAttribute('title')) || squash(el.getAttribute('placeholder'));
}

function tristate(value: string | null): Tristate | undefined {
  if (value === 'mixed') return 'mixed';
  if (value === 'true') return true;
  if (value === 'false') return false;
  return undefined;
}

function checkedOf(el: Element): Tristate | undefined {
  if (el instanceof HTMLInputElement && (el.type === 'checkbox' || el.type === 'radio')) {
    return el.indeterminate ? 'mixed' : el.checked;
  }
  return tristate(el.getAttribute('aria-checked'));
}

function stateOf(el: Element): Omit<Control, keyof ControlRef> {
  const state: Omit<Control, keyof ControlRef> = {};
  if (el.matches(':disabled') || el.getAttribute('aria-disabled') === 'true') state.disabled = true;
  const checked = checkedOf(el);
  if (checked !== undefined) state.checked = checked;
  const pressed = tristate(el.getAttribute('aria-pressed'));
  if (pressed !== undefined) state.pressed = pressed;
  const expanded = tristate(el.getAttribute('aria-expanded'));
  if (typeof expanded === 'boolean') state.expanded = expanded;
  return state;
}

function hidden(el: Element): boolean {
  return el.closest('[hidden], [aria-hidden="true"], [inert]') !== null;
}

function compare(a: Control, b: Control): number {
  if (a.role !== b.role) return a.role < b.role ? -1 : 1;
  if (a.name !== b.name) return a.name < b.name ? -1 : 1;
  const left = JSON.stringify([a.disabled, a.checked, a.pressed, a.expanded]);
  const right = JSON.stringify([b.disabled, b.checked, b.pressed, b.expanded]);
  if (left !== right) return left < right ? -1 : 1;
  return 0;
}

export function controls(root: ParentNode): Control[] {
  const found: Control[] = [];
  for (const el of root.querySelectorAll(CANDIDATES)) {
    if (hidden(el)) continue;
    const role = roleOf(el);
    if (!role || !WIDGET_ROLES.has(role)) continue;
    found.push({ role, name: nameOf(el), ...stateOf(el) });
  }
  return found.sort(compare);
}

export function mergeControls(...lists: Control[][]): Control[] {
  return lists.flat().sort(compare);
}

export function withoutControls(list: Control[], exclude: ControlRef[]): Control[] {
  const rest = [...list];
  for (const control of exclude) {
    const index = rest.findIndex((c) => c.role === control.role && c.name === control.name);
    if (index === -1) throw new Error(`Missing ${control.role} "${control.name}"`);
    rest.splice(index, 1);
  }
  return rest;
}

export function layout<P extends Record<string, unknown>, L extends string>(
  label: L,
  component: Component<P>,
  props: P,
  options: LayoutOptions = {}
): Layout<L> {
  const Host: Component<{ component: Component<P>; props: P }> = ComponentHost;
  return {
    label,
    exclude: options.exclude ?? [],
    open: options.open ?? ((): void => {}),
    render: (target) => {
      const instance = mount(Host, { target, props: { component, props } });
      flushSync();
      return () => unmount(instance);
    }
  };
}

export async function settle(): Promise<void> {
  for (let i = 0; i < 3; i++) {
    await tick();
    await new Promise((resolve) => setTimeout(resolve, 0));
    flushSync();
  }
}

export function renderLayout(entry: Layout): () => Promise<void> {
  const target = document.createElement('div');
  document.body.appendChild(target);
  const destroy = entry.render(target);
  return async () => {
    await destroy();
    target.remove();
  };
}

export async function layoutControls<L extends string>(
  layouts: Layout<L>[]
): Promise<Record<L, Control[]>> {
  const result = {} as Record<L, Control[]>;
  for (const entry of layouts) {
    const target = document.createElement('div');
    document.body.appendChild(target);
    const destroy = entry.render(target);
    try {
      await settle();
      await entry.open(target);
      await settle();
      result[entry.label] = withoutControls(controls(document.body), entry.exclude);
    } finally {
      await destroy();
      flushSync();
      target.remove();
    }
    await settle();
  }
  return result;
}
