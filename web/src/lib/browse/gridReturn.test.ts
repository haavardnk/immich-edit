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
