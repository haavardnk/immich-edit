import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest';
import type { AssetSummary } from '#lib/types/album.ts';

const h = vi.hoisted(() => ({ goto: vi.fn(() => Promise.resolve()) }));
vi.mock('$app/navigation', () => ({ goto: h.goto }));

import { backToGrid } from './backToGrid';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';

function summary(id: string): AssetSummary {
  return { id, originalFileName: `${id}.arw` } as AssetSummary;
}

describe('backToGrid', () => {
  beforeEach(() => {
    h.goto.mockClear();
    browsing.set([summary('a'), summary('b')]);
    browseView.lastGridPath = null;
    browseView.setActive(null);
  });

  it('returns to the grid it came from and selects the open asset', async () => {
    browseView.setLastGridPath('/albums/1?sort=desc');
    await backToGrid('b');
    expect(h.goto).toHaveBeenCalledWith('/albums/1?sort=desc');
    expect(browseView.activeId).toBe('b');
  });

  it('opens a clean photos grid when no grid was visited', async () => {
    browsing.clear();
    await backToGrid('b');
    expect(h.goto).toHaveBeenCalledWith('/photos');
    expect(browseView.activeId).toBeNull();
  });

  it('selects nothing when the asset is not in the loaded list', async () => {
    browseView.setLastGridPath('/photos');
    browseView.setActive('a');
    await backToGrid('c');
    expect(browseView.activeId).toBeNull();
  });

  it('does not reopen the loupe on an asset that left the list', async () => {
    browseView.setLastGridPath('/photos');
    browseView.leaveLoupeForEditor('c');
    await backToGrid('c');
    expect(browseView.loupeId).toBeNull();
  });
});

describe('backToGrid returning', () => {
  const back = vi.fn();

  function stubEntries(urls: string[]): void {
    const index = urls.length - 1;
    Object.defineProperty(globalThis, 'navigation', {
      value: {
        currentEntry: { index, url: `http://localhost${urls[index]}` },
        entries: () => urls.map((url, i) => ({ index: i, url: `http://localhost${url}` }))
      },
      configurable: true
    });
  }

  beforeEach(() => {
    h.goto.mockClear();
    browsing.set([summary('a'), summary('b')]);
    Object.defineProperty(globalThis, 'history', {
      value: { back, length: 2 },
      configurable: true
    });
  });

  afterEach(() => {
    back.mockClear();
    Reflect.deleteProperty(globalThis, 'navigation');
    Reflect.deleteProperty(globalThis, 'history');
  });

  it('pops back to the grid it was pushed from', () => {
    stubEntries(['/albums/x', '/assets/b?from=%2Falbums%2Fx']);
    void backToGrid('b', '/albums/x', 'return');
    expect(back).toHaveBeenCalledOnce();
    expect(h.goto).not.toHaveBeenCalled();
  });

  it('replaces the editor entry when the grid is not the previous entry', async () => {
    stubEntries(['/library', '/assets/b?from=%2Falbums%2Fx']);
    await backToGrid('b', '/albums/x', 'return');
    expect(back).not.toHaveBeenCalled();
    expect(h.goto).toHaveBeenCalledWith('/albums/x', { replace: true });
  });
});
