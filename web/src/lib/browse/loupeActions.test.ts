import type { MetaOutcome } from '#lib/browse/assetMeta.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { compare } from '#lib/stores/compare.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { goto } from '$app/navigation';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import {
  canDropPane,
  leaveLoupeOnEditorPath,
  loupeFocusedId,
  openInEditor,
  rateFocused,
  rateFocusedAndAdvance,
  unflagFocused
} from './loupeActions';

const meta = vi.hoisted(() => ({
  outcome: 'saved' as MetaOutcome,
  setRating: vi.fn(),
  clearFlags: vi.fn()
}));

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/state', () => ({
  page: { url: new URL('http://localhost/albums/a1?sort=asc'), params: {} }
}));
vi.mock('#lib/browse/assetMeta.ts', async (importOriginal) => ({
  ...(await importOriginal<typeof import('#lib/browse/assetMeta.ts')>()),
  setRating: (...args: unknown[]) => {
    meta.setRating(...args);
    return Promise.resolve(meta.outcome);
  },
  clearFlags: (...args: unknown[]) => {
    meta.clearFlags(...args);
    return Promise.resolve(meta.outcome);
  }
}));

function summary(id: string): AssetSummary {
  return { id, originalFileName: `${id}.jpg` } as AssetSummary;
}

async function settle(): Promise<void> {
  await Promise.resolve();
  await Promise.resolve();
}

describe('loupe actions', () => {
  beforeEach(() => {
    vi.mocked(goto).mockClear();
    meta.setRating.mockClear();
    meta.outcome = 'saved';
    browsing.set(['a', 'b', 'c'].map(summary));
    browseView.setLoupeAutoAdvance(false);
    browseView.openLoupe('a');
    compare.exit();
  });

  it('focuses the loupe photo, or the focused pane while comparing', () => {
    expect(loupeFocusedId()).toBe('a');
    compare.enter('compare', ['b', 'c'], 1);
    expect(loupeFocusedId()).toBe('c');
  });

  it('keeps two panes in compare and one in survey', () => {
    compare.enter('compare', ['a', 'b']);
    expect(canDropPane()).toBe(false);
    compare.enter('survey', ['a', 'b']);
    expect(canDropPane()).toBe(true);
  });

  it('advances after rating only when auto-advance is on', async () => {
    rateFocused(3);
    await settle();
    expect(meta.setRating).toHaveBeenCalledWith(expect.anything(), 3);
    expect(browseView.loupeId).toBe('a');

    browseView.setLoupeAutoAdvance(true);
    rateFocused(4);
    await settle();
    expect(browseView.loupeId).toBe('b');
  });

  it('always advances after rate-and-advance unless the write was skipped', async () => {
    meta.outcome = 'skipped';
    rateFocusedAndAdvance(2);
    await settle();
    expect(browseView.loupeId).toBe('a');

    meta.outcome = 'saved';
    rateFocusedAndAdvance(2);
    await settle();
    expect(browseView.loupeId).toBe('b');
  });

  it('never advances after unflagging', async () => {
    browseView.setLoupeAutoAdvance(true);
    unflagFocused();
    await settle();
    expect(meta.clearFlags).toHaveBeenCalledOnce();
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
