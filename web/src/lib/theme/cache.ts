import { isRecord } from '#lib/utils/isRecord.ts';
import { isAppearance } from './appearance';
import { themeCss } from './css';
import { faviconDataUrl } from './favicon';
import type { Appearance, ResolvedTheme, Scheme } from './types';

export const THEME_CACHE_KEY = 'immich-edit:theme';

export const CACHE_VERSION = 1;

export interface SchemeChrome {
  themeColor: string;
  favicon: string;
}

export interface ThemeCache {
  version: typeof CACHE_VERSION;
  revision: string;
  appearance: Appearance;
  css: string;
  chrome: Readonly<Record<Scheme, SchemeChrome>>;
}

function schemeChrome(theme: ResolvedTheme): SchemeChrome {
  return {
    themeColor: theme.palette.base,
    favicon: faviconDataUrl(theme.accentColor)
  };
}

export function buildThemeCache(
  appearance: Appearance,
  themes: Readonly<Record<Scheme, ResolvedTheme>>,
  revision: string
): ThemeCache {
  return {
    version: CACHE_VERSION,
    revision,
    appearance,
    css: themeCss([themes.light, themes.dark]),
    chrome: { light: schemeChrome(themes.light), dark: schemeChrome(themes.dark) }
  };
}

function isChrome(value: unknown): value is SchemeChrome {
  return (
    isRecord(value) && typeof value.themeColor === 'string' && typeof value.favicon === 'string'
  );
}

export function parseThemeCache(value: unknown): ThemeCache | null {
  if (!isRecord(value) || value.version !== CACHE_VERSION) return null;
  if (typeof value.revision !== 'string' || typeof value.css !== 'string') return null;
  if (!isAppearance(value.appearance)) return null;
  if (!isRecord(value.chrome) || !isChrome(value.chrome.light) || !isChrome(value.chrome.dark)) {
    return null;
  }
  return {
    version: CACHE_VERSION,
    revision: value.revision,
    appearance: value.appearance,
    css: value.css,
    chrome: { light: value.chrome.light, dark: value.chrome.dark }
  };
}
