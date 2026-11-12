import type { Keybind, KeybindContext } from './table';

export const CONTEXT_STACKS = {
  masks: ['masks', 'editor', 'global'],
  retouch: ['retouch', 'editor', 'global'],
  geometry: ['geometry', 'editor', 'global'],
  editor: ['editor', 'global'],
  compare: ['compare', 'global'],
  survey: ['survey', 'global'],
  loupe: ['loupe', 'global'],
  grid: ['grid', 'global'],
  viewer: ['viewer']
} as const satisfies Record<string, readonly KeybindContext[]>;

export const CONTEXT_NAMES: Record<KeybindContext, string> = {
  global: 'Anywhere',
  grid: 'Grid',
  loupe: 'Loupe',
  compare: 'Compare',
  survey: 'Survey',
  editor: 'Editor',
  geometry: 'Geometry',
  masks: 'Masks',
  retouch: 'Retouch',
  viewer: 'Viewer window'
};

const STACKS: readonly (readonly KeybindContext[])[] = Object.values(CONTEXT_STACKS);

export function sharedContext(a: Keybind, b: Keybind): KeybindContext | null {
  for (const stack of STACKS) {
    if (!a.contexts.some((context) => stack.includes(context))) continue;
    const shared = b.contexts.find((context) => stack.includes(context));
    if (shared) return shared;
  }
  return null;
}
