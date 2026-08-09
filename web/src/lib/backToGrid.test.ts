import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { AssetSummary } from '$lib/types/album';

const h = vi.hoisted(() => ({ goto: vi.fn(() => Promise.resolve()) }));
vi.mock('$app/navigation', () => ({ goto: h.goto }));

import { backToGrid } from './backToGrid';
import { browseView } from './stores/browseView.svelte';
import { browsing } from './stores/browsing.svelte';

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
});
