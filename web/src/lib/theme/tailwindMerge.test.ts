import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { TYPE_SCALE, twMerge } from './tailwindMerge.ts';

describe('scale-aware class merge', () => {
  it('treats every type-scale step as a size, not a colour', () => {
    for (const step of TYPE_SCALE) {
      expect(twMerge(`text-sm text-light text-${step}`)).toBe(`text-light text-${step}`);
    }
  });

  it('knows every size token the theme defines', () => {
    const css = readFileSync(new URL('../../app.css', import.meta.url), 'utf8');
    const defaults = new Set(['xs', 'sm', 'base', 'lg', 'xl', '2xl']);
    const tokens = [...css.matchAll(/--text-([a-z0-9]+):/g)]
      .map(([, name = '']) => name)
      .filter((name) => !defaults.has(name));
    expect(tokens.sort()).toEqual([...TYPE_SCALE].sort());
  });
});
