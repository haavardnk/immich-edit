import { beforeEach, describe, expect, it } from 'vitest';
import { selection } from '#lib/stores/selection.svelte.ts';
import { asset } from '#lib/testing/assets.ts';
import { GridSelectAll } from './gridSelectAll.svelte';

function grid(loads: boolean, loadingMore = false) {
  let items = ['a', 'b'].map((id) => asset(id));
  const whole = new GridSelectAll({
    items: () => items,
    loadingMore: () => loadingMore,
    loadAll: () => async () => {
      expect(whole.busy).toBe(true);
      if (loads) items = ['a', 'b', 'c', 'd'].map((id) => asset(id));
      return loads;
    }
  });
  return whole;
}

beforeEach(() => selection.clear());

describe('GridSelectAll', () => {
  it('loads the whole view before selecting all of it', async () => {
    const whole = grid(true);
    expect(await whole.run()).toBe(true);
    expect([...selection.selected]).toEqual(['a', 'b', 'c', 'd']);
    expect(whole.busy).toBe(false);
  });

  it('keeps the selection when the view fails to load', async () => {
    selection.replace(['b']);
    expect(await grid(false).run()).toBe(false);
    expect([...selection.selected]).toEqual(['b']);
  });

  it('stops loading and keeps the selection when cancelled', async () => {
    selection.replace(['b']);
    const signals: AbortSignal[] = [];
    let finish = (_loaded: boolean): void => {};
    const whole = new GridSelectAll({
      items: () => ['a', 'b', 'c'].map((id) => asset(id)),
      loadingMore: () => false,
      loadAll: () => (next) => {
        signals.push(next);
        return new Promise<boolean>((resolve) => (finish = resolve));
      }
    });

    const running = whole.run();
    whole.cancel();
    expect(whole.busy).toBe(false);
    expect(signals[0]?.aborted).toBe(true);
    finish(true);
    expect(await running).toBe(false);
    expect([...selection.selected]).toEqual(['b']);
    expect(whole.busy).toBe(false);
  });

  it('waits for a page that is already loading', async () => {
    selection.replace(['b']);
    expect(await grid(true, true).run()).toBe(false);
    expect([...selection.selected]).toEqual(['b']);
  });
});
