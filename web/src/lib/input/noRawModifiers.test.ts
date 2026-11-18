import { describe, expect, it } from 'vitest';

const sources = import.meta.glob<string>('/src/**/*.{ts,svelte}', {
  query: '?raw',
  import: 'default',
  eager: true
});

const OWNERS = ['/src/lib/input/', '/src/lib/shortcuts/', '/src/lib/commands/'];

function readsModifiers(source: string): boolean {
  return /\b(shift|alt|ctrl|meta)Key\b|\bgetModifierState\(/.test(source);
}

function offenders(): string[] {
  return Object.entries(sources)
    .filter(([path]) => !path.endsWith('.test.ts'))
    .filter(([path]) => !OWNERS.some((owner) => path.startsWith(owner)))
    .filter(([, source]) => readsModifiers(source))
    .map(([path]) => path)
    .sort();
}

describe('modifier reads', () => {
  it('reads modifier keys only through the input roles', () => {
    expect(Object.keys(sources).length).toBeGreaterThan(100);
    expect(offenders()).toEqual([]);
  });

  it.each([
    ['a shift read', 'const step = e.shiftKey ? 10 : 1;', true],
    ['an alt read', 'if (e.altKey) sample();', true],
    ['a ctrl or meta read', 'if (e.ctrlKey || e.metaKey) zoom();', true],
    ['a modifier state read', "if (e.getModifierState('Shift')) extend();", true],
    ['a role read', "const step = held('coarse', e) ? 10 : 1;", false],
    ['a held-shift flag', 'let shiftPressed = $state(false);', false]
  ])('classifies %s', (_, source, flagged) => {
    expect(readsModifiers(source)).toBe(flagged);
  });
});
