import { describe, expect, it } from 'vitest';
import { findPage, legacyTarget, settingsHref, visiblePages } from './pages';

describe('settings pages', () => {
  it('hides Administration from other users', () => {
    expect(visiblePages(false).map((page) => page.id)).toEqual([
      'preferences',
      'account',
      'diagnostics'
    ]);
    expect(visiblePages(true)).toHaveLength(4);
  });

  it('sends old section links to the row on its new page', () => {
    expect(legacyTarget('models')).toEqual({ page: 'administration', anchor: 'models' });
    expect(legacyTarget('sessions')).toEqual({ page: 'account', anchor: 'sessions' });
    expect(legacyTarget('account')).toEqual({ page: 'account' });
    expect(legacyTarget('nope')).toBeNull();
  });

  it('builds page links with an optional anchor', () => {
    expect(settingsHref('preferences')).toBe('/settings/preferences');
    expect(settingsHref('administration', 'lens-auto')).toBe('/settings/administration#lens-auto');
    expect(findPage('diagnostics')?.title).toBe('Diagnostics');
    expect(findPage(undefined)).toBeNull();
  });
});
