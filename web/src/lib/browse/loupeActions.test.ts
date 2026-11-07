import type { MetaOutcome, MetaTarget } from '#lib/browse/assetMeta.ts';
import { ADVANCE_MS, ratingHud } from '#lib/browse/ratingHud.svelte.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { compare } from '#lib/stores/compare.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import type { ExifInfo } from '#lib/types/asset.ts';
import { goto } from '$app/navigation';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
  canDropPane,
  dropFocusedPane,
  leaveLoupeOnEditorPath,
  loupeFocusedId,
  openInEditor,
  rateFocused,
  rateFocusedAndAdvance,
  rejectFocused,
  unflagFocused
} from './loupeActions';

const meta = vi.hoisted(() => ({
  outcome: 'saved' as MetaOutcome,
  setRating: vi.fn(),
  toggleReject: vi.fn(),
  clearFlags: vi.fn()
}));

function written(target: MetaTarget, fields: Parameters<MetaTarget['patch']>[0]): MetaOutcome {
  if (meta.outcome !== 'skipped') target.patch(fields);
  return meta.outcome;
}

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/state', () => ({
  page: { url: new URL('http://localhost/albums/a1?sort=asc'), params: {} }
}));
vi.mock('#lib/browse/assetMeta.ts', async (importOriginal) => ({
  ...(await importOriginal<typeof import('#lib/browse/assetMeta.ts')>()),
  setRating: (target: MetaTarget, rating: number | null) => {
    meta.setRating(target, rating);
    return Promise.resolve(written(target, { exifInfo: { rating } as ExifInfo }));
  },
  toggleReject: (target: MetaTarget) => {
    meta.toggleReject(target);
    return Promise.resolve(written(target, {}));
  },
  clearFlags: (target: MetaTarget) => {
    meta.clearFlags(target);
    return Promise.resolve(written(target, { isFavorite: false }));
  }
}));

function summary(id: string): AssetSummary {
  return {
    id,
    originalFileName: `${id}.jpg`,
    type: 'IMAGE',
    fileCreatedAt: null,
    updatedAt: null,
    checksum: null,
    isFavorite: false,
    exifInfo: null,
    tags: []
  };
}

async function settle(): Promise<void> {
  await Promise.resolve();
  await Promise.resolve();
}

describe('loupe actions', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    ratingHud.clear();
    vi.mocked(goto).mockClear();
    meta.setRating.mockClear();
    meta.clearFlags.mockClear();
    meta.outcome = 'saved';
    browsing.set(['a', 'b', 'c'].map(summary));
    browseView.setLoupeAutoAdvance(false);
    browseView.openLoupe('a');
    compare.exit();
  });

  afterEach(() => {
    ratingHud.clear();
    vi.useRealTimers();
  });

  it('focuses the loupe photo, or the focused pane while comparing', () => {
    expect(loupeFocusedId()).toBe('a');
    compare.enter('compare', ['b', 'c'], 1);
    expect(loupeFocusedId()).toBe('c');
  });

  it('drops a pane while more than one remains, leaving two panes for the survivor', () => {
    compare.enter('compare', ['a', 'b'], 1);
    expect(canDropPane()).toBe(true);
    dropFocusedPane();
    expect(compare.mode).toBe('single');
    expect(browseView.loupeId).toBe('a');
    compare.enter('survey', ['a', 'b', 'c']);
    dropFocusedPane();
    expect(compare.members).toEqual(['b', 'c']);
    compare.keepOnly(0);
    expect(canDropPane()).toBe(false);
  });

  it('shows the rating and advances after the countdown only when auto-advance is on', async () => {
    rateFocused(3);
    await settle();
    expect(meta.setRating).toHaveBeenCalledWith(expect.anything(), 3);
    expect(ratingHud.shown).toMatchObject({ id: 'a', kind: 'rating', advancing: false });
    expect(ratingHud.shown?.state.rating).toBe(3);
    vi.advanceTimersByTime(ADVANCE_MS * 4);
    expect(browseView.loupeId).toBe('a');
    expect(ratingHud.shown).toBeNull();

    browseView.setLoupeAutoAdvance(true);
    rateFocused(4);
    await settle();
    expect(ratingHud.shown).toMatchObject({ advancing: true });
    expect(browseView.loupeId).toBe('a');
    vi.advanceTimersByTime(ADVANCE_MS);
    expect(browseView.loupeId).toBe('b');
    vi.advanceTimersByTime(ADVANCE_MS * 4);
    expect(browseView.loupeId).toBe('b');
  });

  it('always advances after rate-and-advance unless the write was skipped', async () => {
    meta.outcome = 'skipped';
    rateFocusedAndAdvance(2);
    await settle();
    expect(ratingHud.shown).toBeNull();
    vi.advanceTimersByTime(ADVANCE_MS);
    expect(browseView.loupeId).toBe('a');

    meta.outcome = 'saved';
    rateFocusedAndAdvance(2);
    await settle();
    expect(ratingHud.shown).toMatchObject({ advancing: true });
    vi.advanceTimersByTime(ADVANCE_MS);
    expect(browseView.loupeId).toBe('b');
  });

  it('shows flags only when they advance', async () => {
    rejectFocused();
    await settle();
    expect(ratingHud.shown).toBeNull();

    browseView.setLoupeAutoAdvance(true);
    rejectFocused();
    await settle();
    expect(ratingHud.shown).toMatchObject({ kind: 'reject', advancing: true });
    vi.advanceTimersByTime(ADVANCE_MS);
    expect(browseView.loupeId).toBe('b');
  });

  it('stays on a photo opened elsewhere during the countdown', async () => {
    browseView.setLoupeAutoAdvance(true);
    rateFocused(4);
    await settle();
    expect(ratingHud.shown).toMatchObject({ advancing: true });
    browseView.openLoupe('b');
    vi.advanceTimersByTime(ADVANCE_MS);
    expect(browseView.loupeId).toBe('b');
  });

  it('never advances after unflagging', async () => {
    browseView.setLoupeAutoAdvance(true);
    unflagFocused();
    await settle();
    expect(meta.clearFlags).toHaveBeenCalledOnce();
    expect(ratingHud.shown).toBeNull();
    vi.advanceTimersByTime(ADVANCE_MS * 4);
    expect(browseView.loupeId).toBe('a');
  });

  it('opens the focused photo in the editor with the current page to return to', () => {
    openInEditor();
    expect(goto).toHaveBeenCalledWith('/assets/a?from=%2Falbums%2Fa1%3Fsort%3Dasc');
    expect(browseView.loupeId).toBeNull();
  });

  it.each([
    ['the editor closes', '/assets/a', null, true],
    ['a browse page keeps', '/albums/a1', 'a', false]
  ])('navigating to %s the loupe', (_, path, open, returnsToLoupe) => {
    browseView.consumeEditorReturnLoupe(null);
    leaveLoupeOnEditorPath(path);
    expect(browseView.loupeId).toBe(open);
    expect(browseView.consumeEditorReturnLoupe('a')).toBe(returnsToLoupe);
  });
});
