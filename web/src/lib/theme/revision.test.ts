import { describe, expect, it } from 'vitest';
import { FAMILIES } from './registry';
import { themeRevision } from './revision';
import type { ThemeFamily } from './types';

function withBase(family: ThemeFamily, base: string): ThemeFamily {
  const [first, ...rest] = family.themes;
  if (!first) throw new Error(`${family.id} has no themes`);
  return { ...family, themes: [{ ...first, palette: { ...first.palette, base } }, ...rest] };
}

describe('theme revision', () => {
  const revision = themeRevision(FAMILIES);

  it('is a stable short hash of the built-in themes', () => {
    expect(revision).toMatch(/^[0-9a-f]{8}$/);
    expect(themeRevision(FAMILIES)).toBe(revision);
  });

  it('changes when any palette changes', () => {
    const [first, ...rest] = FAMILIES;
    if (!first) throw new Error('no families registered');
    expect(themeRevision([withBase(first, '#010203'), ...rest])).not.toBe(revision);
    expect(themeRevision(rest)).not.toBe(revision);
  });
});
