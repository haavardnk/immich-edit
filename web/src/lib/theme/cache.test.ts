import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { buildThemeCache, parseThemeCache } from './cache';
import { FAVICON_ACCENT, faviconDataUrl, faviconSvg } from './favicon';
import { DEFAULT_APPEARANCE, resolveAppearance } from './registry';

describe('theme cache', () => {
  const cache = buildThemeCache(DEFAULT_APPEARANCE, resolveAppearance(DEFAULT_APPEARANCE), 'r1');

  it('round-trips through JSON', () => {
    expect(parseThemeCache(JSON.parse(JSON.stringify(cache)))).toEqual(cache);
  });

  it('carries the bar colour and an accent favicon per scheme', () => {
    expect(cache.chrome.dark.themeColor).toBe('#000000');
    expect(cache.chrome.light.themeColor).toBe('#ffffff');
    expect(cache.chrome.dark.favicon).toBe(faviconDataUrl('#b7a1fb'));
    expect(cache.chrome.light.favicon).toBe(faviconDataUrl('#7159b0'));
  });

  it('keeps an entry from another revision so the stored choice survives an upgrade', () => {
    expect(parseThemeCache({ ...cache, revision: 'r0' })?.appearance).toEqual(DEFAULT_APPEARANCE);
  });

  it('rejects missing, malformed and other-version entries', () => {
    expect(parseThemeCache(null)).toBeNull();
    expect(parseThemeCache({ ...cache, version: 2 })).toBeNull();
    expect(parseThemeCache({ ...cache, revision: undefined })).toBeNull();
    expect(
      parseThemeCache({ ...cache, appearance: { ...cache.appearance, mode: 'dim' } })
    ).toBeNull();
    expect(parseThemeCache({ ...cache, chrome: { light: cache.chrome.light } })).toBeNull();
  });
});

describe('favicon', () => {
  it('matches the static favicon served before the bundle loads', () => {
    const staticSvg = readFileSync(
      fileURLToPath(new URL('../../../static/favicon.svg', import.meta.url)),
      'utf8'
    );
    expect(faviconSvg(FAVICON_ACCENT)).toBe(staticSvg);
  });

  it('recolours the static mark with the accent and keeps the OS-following ink', () => {
    const svg = decodeURIComponent(faviconDataUrl('#123456').replace('data:image/svg+xml,', ''));
    expect(svg).toContain('stroke="#123456"');
    expect(svg).toContain('prefers-color-scheme: dark');
    expect(svg).not.toContain(FAVICON_ACCENT);
  });
});
