import type { MetaOutcome } from '#lib/browse/assetMeta.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { compare } from '#lib/stores/compare.svelte.ts';
import { labelTagValue } from '#lib/stores/labels.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import type { TagRef } from '#lib/types/asset.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import {
  gridCurrent,
  nextTargetLabel,
  nextTargetRating,
  openMulti,
  rateAndAdvance
} from './gridActions';

const meta = vi.hoisted(() => ({ outcome: 'saved' as MetaOutcome, setRating: vi.fn() }));

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('#lib/browse/assetMeta.ts', async (importOriginal) => ({
  ...(await importOriginal<typeof import('#lib/browse/assetMeta.ts')>()),
  setRating: (...args: unknown[]) => {
    meta.setRating(...args);
    return Promise.resolve(meta.outcome);
  }
}));

const RED = { id: 'red', value: labelTagValue('red'), name: 'red' } as TagRef;

function summary(id: string, rating: number | null = null, tags: TagRef[] = []): AssetSummary {
  return { id, originalFileName: `${id}.jpg`, exifInfo: { rating }, tags } as AssetSummary;
}

const ITEMS = [summary('a', 3), summary('b', 3), summary('c', 5)];

async function settle(): Promise<void> {
  await Promise.resolve();
  await Promise.resolve();
}

describe('grid actions', () => {
  beforeEach(() => {
    meta.setRating.mockClear();
    meta.outcome = 'saved';
    selection.clear();
    browseView.setActive(null);
    browseView.closeLoupe();
  });

  it('prefers the active photo when it is selected, else the first selected one', () => {
    selection.replace(['b', 'c']);
    expect(gridCurrent(ITEMS)).toBe('b');
    browseView.setActive('c');
    expect(gridCurrent(ITEMS)).toBe('c');
    browseView.setActive('a');
    expect(gridCurrent(ITEMS)).toBe('b');
  });

  it('toggles a shared rating and treats a mixed one as unset', () => {
    selection.replace(['a', 'b']);
    expect(nextTargetRating(ITEMS, '3')).toBeNull();
    selection.replace(['a', 'c']);
    expect(nextTargetRating(ITEMS, '3')).toBe(3);
  });

  it('offers no label without targets and clears a shared one', () => {
    const items = [summary('a', null, [RED]), summary('b', null, [RED])];
    expect(nextTargetLabel(items, '6')).toBeUndefined();
    selection.replace(['a', 'b']);
    expect(nextTargetLabel(items, '6')).toBeNull();
  });

  it('moves a single selection to the next photo after rating it', async () => {
    const reveal = vi.fn();
    selection.replace(['a']);
    rateAndAdvance(ITEMS, 4, reveal);
    await settle();
    expect(meta.setRating).toHaveBeenCalledOnce();
    expect([...selection.selected]).toEqual(['b']);
    expect(browseView.activeId).toBe('b');
    expect(reveal).toHaveBeenCalledWith('b');
  });

  it('rates a multi-selection without moving it', async () => {
    const reveal = vi.fn();
    selection.replace(['a', 'b']);
    rateAndAdvance(ITEMS, 4, reveal);
    await settle();
    expect(meta.setRating).toHaveBeenCalledTimes(2);
    expect([...selection.selected]).toEqual(['a', 'b']);
    expect(reveal).not.toHaveBeenCalled();
  });

  it('opens a comparison of the selection in the loupe', () => {
    selection.replace(['a', 'c']);
    openMulti('compare', ITEMS);
    expect(browseView.loupeId).toBe('a');
    expect(compare.mode).toBe('compare');
    expect(compare.members).toEqual(['a', 'c']);
  });
});
