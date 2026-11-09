import { describe, expect, it } from 'vitest';
import { NAV_TABS, type NavTab } from './navTabs';
import { tabMemory } from './tabMemory.svelte';

function tab(id: NavTab['id']): NavTab {
  const found = NAV_TABS.find((candidate) => candidate.id === id);
  if (!found) throw new Error(id);
  return found;
}

function at(href: string): URL {
  return new URL(href, 'http://app.invalid');
}

describe('tabMemory', () => {
  it('returns to the last page of another tab and to the root of the current one', () => {
    tabMemory.visit(at('/albums'));
    tabMemory.visit(at('/albums/trip'));
    tabMemory.visit(at('/photos?rating=3'));

    expect(tabMemory.href(tab('albums'), at('/photos?rating=3'))).toBe('/albums/trip');
    expect(tabMemory.href(tab('photos'), at('/albums/trip'))).toBe('/photos?rating=3');
    expect(tabMemory.href(tab('albums'), at('/albums/trip'))).toBe('/albums');
    expect(tabMemory.href(tab('photos'), at('/photos?rating=3'))).toBe('/photos?rating=3');
    expect(tabMemory.href(tab('search'), at('/photos'))).toBe('/search');
  });

  it('keeps library pages under the library tab', () => {
    tabMemory.visit(at('/favorites'));
    expect(tabMemory.href(tab('library'), at('/photos'))).toBe('/favorites');
    expect(tabMemory.href(tab('library'), at('/favorites'))).toBe('/library');
  });
});
