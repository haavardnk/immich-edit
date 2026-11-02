import { readStored, writeStored } from '#lib/utils/storage.ts';
import { parseThemeCache, THEME_CACHE_KEY, type SchemeChrome, type ThemeCache } from './cache';

export const THEME_STYLE_ID = 'immich-edit-theme';

function themeStyle(): HTMLStyleElement {
  const existing = document.getElementById(THEME_STYLE_ID);
  if (existing instanceof HTMLStyleElement) return existing;
  const style = document.createElement('style');
  style.id = THEME_STYLE_ID;
  document.head.append(style);
  return style;
}

function headElement<T extends HTMLElement>(selector: string, create: () => T): T {
  const existing = document.head.querySelector<T>(selector);
  if (existing) return existing;
  const element = create();
  document.head.append(element);
  return element;
}

export function pageThemeRevision(): string {
  return document.documentElement.dataset.themeRevision ?? '';
}

export function readThemeCache(): ThemeCache | null {
  return parseThemeCache(readStored<unknown>(THEME_CACHE_KEY));
}

export function writeTheme(cache: ThemeCache): void {
  themeStyle().textContent = cache.css;
  document.documentElement.dataset.theme = '';
  writeStored(THEME_CACHE_KEY, cache);
}

export function writeSchemeChrome(chrome: SchemeChrome): void {
  headElement('meta[name="theme-color"]', () => {
    const meta = document.createElement('meta');
    meta.name = 'theme-color';
    return meta;
  }).content = chrome.themeColor;
  headElement('link[rel="icon"]', () => {
    const link = document.createElement('link');
    link.rel = 'icon';
    link.type = 'image/svg+xml';
    return link;
  }).href = chrome.favicon;
}
