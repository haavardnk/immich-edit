import { describe, expect, it } from 'vitest';
import { findPage, settingsHref, visiblePages } from './pages';

describe('settings pages', () => {
  it('hides Administration from other users', () => {
    expect(visiblePages(false).map((page) => page.id)).toEqual([
      'preferences',
      'account',
      'diagnostics'
    ]);
    expect(visiblePages(true)).toHaveLength(4);
  });

  it('builds page links with an optional anchor', () => {
    expect(settingsHref('preferences')).toBe('/settings/preferences');
    expect(settingsHref('administration', 'lens-auto')).toBe('/settings/administration#lens-auto');
    expect(findPage('diagnostics')?.title).toBe('Diagnostics');
    expect(findPage(undefined)).toBeNull();
  });
});
