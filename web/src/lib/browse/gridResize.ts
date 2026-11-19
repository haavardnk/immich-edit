import { gridSizeHud } from '#lib/browse/gridSizeHud.svelte.ts';
import { steppedGridSize, type GridSize } from '#lib/browse/gridSizes.ts';
import { gridHost } from '#lib/commands/grid/gridHost.ts';
import type { Point } from '#lib/gestures/point.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';

export type GridResizeOrigin = { anchor?: Point; hud?: Point };

export function resizeGrid(size: GridSize, origin: GridResizeOrigin = {}): void {
  const settle =
    size === browseView.gridSize ? null : gridHost.get()?.holdAnchor(origin.anchor ?? null);
  browseView.setGridSize(size);
  settle?.();
  gridSizeHud.show(origin.hud ?? null);
}

export function stepGrid(delta: number, origin: GridResizeOrigin = {}): void {
  if (delta === 0) return;
  resizeGrid(steppedGridSize(browseView.gridSize, delta), origin);
}

export function clickOrigin(
  e: Pick<MouseEvent, 'detail' | 'clientX' | 'clientY'>
): GridResizeOrigin {
  return e.detail === 0 ? {} : { hud: { x: e.clientX, y: e.clientY } };
}
