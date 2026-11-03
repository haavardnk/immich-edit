import { describe, expect, it } from 'vitest';

const sources = import.meta.glob<string>('/src/**/*.{ts,svelte}', {
  query: '?raw',
  import: 'default',
  eager: true
});

const OWNERS = ['/src/lib/commands/', '/src/lib/shortcuts/'];

const AWAITING = [
  '/src/lib/components/browse/AssetGrid.svelte',
  '/src/lib/components/browse/Loupe.svelte',
  '/src/lib/components/browse/filters/FilterPanel.svelte'
];

function dispatchesBinds(source: string): boolean {
  return /\bmatchKeybind\s*\(/.test(source);
}

describe('shortcut dispatch', () => {
  it('resolves shortcuts only through the command registry', () => {
    expect(Object.keys(sources).length).toBeGreaterThan(100);
    const offenders = Object.entries(sources)
      .filter(([path]) => !path.endsWith('.test.ts'))
      .filter(([path]) => !OWNERS.some((owner) => path.startsWith(owner)))
      .filter(([, source]) => dispatchesBinds(source))
      .map(([path]) => path)
      .sort();
    expect(offenders).toEqual(AWAITING);
  });

  it.each([
    ['a bind lookup', 'const bind = matchKeybind(e, contexts);', true],
    ['a single chord check', "if (isKeybind(e, 'maskDelete')) remove();", false]
  ])('flags %s', (_, source, flagged) => {
    expect(dispatchesBinds(source)).toBe(flagged);
  });
});
