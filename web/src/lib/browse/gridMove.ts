import { flushSync } from 'svelte';
import type { AssetGridLayout } from '#lib/browse/assetGridLayout.ts';
import { gridCurrent } from '#lib/browse/gridActions.ts';
import { moveIndex, pageRows } from '#lib/browse/gridKeys.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';

export type GridMoveSource = {
  items: () => readonly AssetSummary[];
  layout: () => AssetGridLayout;
  viewHeight: () => number;
  reveal: (id: string) => void;
};

export function gridMover(src: GridMoveSource): (key: string, extend: boolean) => void {
  function land(target: AssetSummary | undefined, extend: boolean): void {
    if (!target) return;
    const items = src.items();
    if (extend && selection.active) {
      selection.range(
        items.map((asset) => asset.id),
        target.id
      );
    } else {
      selection.selectLoaded([target.id]);
    }
    browseView.setActive(target.id);
    src.reveal(target.id);
  }

  function move(key: string, extend: boolean, retried: boolean): void {
    const items = src.items();
    const cursorId = selection.active ? gridCurrent(items) : browseView.activeId;
    const target = moveIndex(key, {
      layout: src.layout(),
      count: items.length,
      cursor: cursorId ? items.findIndex((asset) => asset.id === cursorId) : -1,
      selecting: selection.active,
      pageRows: pageRows(src.viewHeight(), browseView.rowHeight),
      hasPrevious: browsing.hasPrevious,
      hasMore: browsing.hasMore
    });
    if (target.kind === 'index') {
      land(items[Math.min(items.length - 1, target.index)], extend);
      return;
    }
    if (target.kind === 'edge') {
      void browsing.requestEdge(target.end).then(() => {
        flushSync();
        const list = src.items();
        land(target.end === 'first' ? list[0] : list.at(-1), extend);
      });
      return;
    }
    if (retried) return;
    const request = target.end === 'first' ? browsing.requestPrevious() : browsing.requestMore();
    void request.then((loaded) => {
      if (!loaded) return;
      flushSync();
      move(key, extend, true);
    });
  }

  return (key, extend) => move(key, extend, false);
}
