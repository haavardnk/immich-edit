import { describe, expect, it } from 'vitest';
import { searchSettings } from './searchSettings';
import { SETTINGS_INDEX } from './settingsIndex';
import { SETTINGS_PAGES, visiblePages } from './pages';

const ADMIN = { pages: SETTINGS_PAGES, desktop: true, installable: true };

function titles(query: string, scope = ADMIN): string[] {
  return searchSettings(query, SETTINGS_INDEX, scope).map((hit) => hit.entry.title);
}

describe('searchSettings', () => {
  it('finds a row by a word and links to its page and anchor', () => {
    const [hit] = searchSettings('lens', SETTINGS_INDEX, ADMIN);
    expect(hit?.entry.title).toBe('Automatic lens corrections');
    expect(hit?.page.id).toBe('administration');
    expect(hit?.href).toBe('/settings/administration#lens-auto');
  });

  it('matches every word in any order, ignoring case and accents', () => {
    expect(titles('RENDER latency')).toEqual(['Render latency']);
    expect(titles('rose pine')).toEqual(['Themes and accents']);
    expect(titles('   ')).toEqual([]);
  });

  it('leaves out admin pages, desktop-only rows on touch and install when unavailable', () => {
    const user = { pages: visiblePages(false), desktop: false, installable: false };
    expect(titles('users', user)).toEqual([]);
    expect(titles('install', user)).toEqual([]);
    expect(titles('loupe', user)).toEqual([]);
    expect(titles('loupe')).toEqual(['Click a photo']);
  });

  it('points every entry at a known page', () => {
    const ids = new Set(SETTINGS_PAGES.map((page) => page.id));
    for (const entry of SETTINGS_INDEX) expect(ids.has(entry.page)).toBe(true);
  });
});
