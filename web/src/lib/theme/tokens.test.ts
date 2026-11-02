import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { themeCss } from './css';
import { DEFAULT_APPEARANCE, resolveAppearance } from './registry';

const appCss = readFileSync(fileURLToPath(new URL('../../app.css', import.meta.url)), 'utf8');

function fallbackBlock(scheme: 'light' | 'dark'): Record<string, string> {
  const selector = `:root.${scheme}:not([data-theme]) {`;
  const start = appCss.indexOf(selector);
  expect(start, `app.css has no ${selector} block`).toBeGreaterThanOrEqual(0);
  const body = appCss.slice(start + selector.length, appCss.indexOf('}', start));
  return Object.fromEntries(
    [...body.matchAll(/(--[\w-]+):\s*([^;]+);/g)].map((match) => [match[1], match[2]?.trim()])
  );
}

describe('theme tokens', () => {
  const themes = resolveAppearance(DEFAULT_APPEARANCE);

  it('keep the app.css first-paint fallback equal to immich-edit violet', () => {
    expect(fallbackBlock('dark')).toEqual(themes.dark.tokens);
    expect(fallbackBlock('light')).toEqual(themes.light.tokens);
  });

  it('leave inherited groups to the package defaults', () => {
    const names = Object.keys(themes.dark.tokens);
    expect(names.some((name) => name.startsWith('--immich-ui-primary-'))).toBe(true);
    expect(names.some((name) => name.startsWith('--color-gray-'))).toBe(true);
    expect(names.some((name) => name.startsWith('--immich-ui-light-'))).toBe(false);
    expect(names).not.toContain('--color-editor-chrome');
  });

  it('render one block per scheme', () => {
    const css = themeCss([themes.light, themes.dark]);
    expect(css.startsWith(':root.light{--immich-ui-primary-50:#f3f1fc;')).toBe(true);
    expect(css).toContain('}:root.dark{--immich-ui-primary-50:#221533;');
    expect(css.endsWith('--color-gray-950:#120f15;}')).toBe(true);
  });
});
