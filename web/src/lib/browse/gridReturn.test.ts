import { afterEach, describe, expect, it } from 'vitest';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { gridPath, resumeId } from './gridReturn';

const photos = new URL('http://app.test/photos?rating=3');

afterEach(() => {
  browseView.lastGridPath = null;
  browseView.setActive(null);
  browseView.setGridAnchor(gridPath(photos), null);
});

describe('resumeId', () => {
  it('starts at the top of a grid never left mid-way', () => {
    expect(resumeId(photos)).toBeNull();
  });

  it('returns to the active photo of the grid that was left last', () => {
    browseView.setLastGridPath('/photos?rating=3');
    browseView.setActive('p1');
    browseView.setGridAnchor('/photos?rating=3', { id: 'top', index: 4, ratio: 0, offset: 0 });
    expect(resumeId(photos)).toBe('p1');
  });

  it('falls back to the remembered top photo of that grid', () => {
    browseView.setLastGridPath('/favorites');
    browseView.setActive('p1');
    browseView.setGridAnchor('/photos?rating=3', { id: 'top', index: 4, ratio: 0, offset: 0 });
    expect(resumeId(photos)).toBe('top');
  });
});

describe('gridPath', () => {
  it('keeps the map list the same grid wherever the camera is', () => {
    expect(gridPath(new URL('http://app.test/map?at=60.1,6.7,12.00&favorite=true'))).toBe(
      '/map?favorite=true'
    );
    expect(gridPath(new URL('http://app.test/map?at=10.8,41.2,1.93'))).toBe('/map');
    expect(gridPath(new URL('http://app.test/search?q=a%20b'))).toBe('/search?q=a%20b');
  });
});
