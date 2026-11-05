import { describe, expect, it } from 'vitest';

const sources = import.meta.glob<string>('/src/**/*.{ts,svelte}', {
  query: '?raw',
  import: 'default',
  eager: true
});

const OWNERS = ['/src/lib/gestures/'];

function capturesPointer(source: string): boolean {
  return /\b(set|release|has)PointerCapture\b/.test(source);
}

function offenders(): string[] {
  return Object.entries(sources)
    .filter(([path]) => !path.endsWith('.test.ts'))
    .filter(([path]) => !OWNERS.some((owner) => path.startsWith(owner)))
    .filter(([, source]) => capturesPointer(source))
    .map(([path]) => path)
    .sort();
}

describe('pointer capture', () => {
  it('captures pointers only through the shared drag', () => {
    expect(Object.keys(sources).length).toBeGreaterThan(100);
    expect(offenders()).toEqual([]);
  });

  it.each([
    ['a capture', 'el.setPointerCapture(e.pointerId);', true],
    ['a release', 'el.releasePointerCapture?.(e.pointerId);', true],
    ['a capture check', 'if (el.hasPointerCapture(id)) stop();', true],
    ['a lost capture handler', 'onlostpointercapture={drag.follow.onlostpointercapture}', false],
    ['a shared drag', '{...drag.on()}', false]
  ])('classifies %s', (_, source, flagged) => {
    expect(capturesPointer(source)).toBe(flagged);
  });
});
