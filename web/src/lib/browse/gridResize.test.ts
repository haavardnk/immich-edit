import { afterEach, describe, expect, it, vi } from 'vitest';
import { gridHost, type GridHost } from '#lib/commands/grid/gridHost.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { gridSizeHud } from './gridSizeHud.svelte';
import { clickOrigin, resizeGrid, stepGrid } from './gridResize';

vi.mock('#lib/device/layout.svelte.ts', async () => ({
  layout: (await import('#lib/testing/layoutMock.ts')).fakeLayout
}));

const calls: string[] = [];
let detach = (): void => {};

function attachHost(): GridHost['holdAnchor'] {
  const holdAnchor = vi.fn((at: { x: number; y: number } | null) => {
    calls.push(`hold ${browseView.gridSize} ${at ? `${at.x},${at.y}` : 'none'}`);
    return () => calls.push(`settle ${browseView.gridSize}`);
  });
  detach = gridHost.attach({
    items: () => [],
    selectingAll: () => false,
    selectAll: async () => true,
    move: () => {},
    reveal: () => {},
    scrollTo: () => {},
    focusBulkBar: () => {},
    holdAnchor
  });
  return holdAnchor;
}

afterEach(() => {
  detach();
  calls.length = 0;
  browseView.setGridSize('md');
  gridSizeHud.hide();
});

describe('grid resize', () => {
  it('holds the anchor around the size change and shows the HUD at its point', () => {
    attachHost();

    stepGrid(1, { anchor: { x: 5, y: 6 }, hud: { x: 7, y: 8 } });

    expect(calls).toEqual(['hold md 5,6', 'settle lg']);
    expect(browseView.gridSize).toBe('lg');
    expect(gridSizeHud.visible).toBe(true);
    expect(gridSizeHud.at).toEqual({ x: 7, y: 8 });
  });

  it('shows the HUD without moving the grid when the size does not change', () => {
    const holdAnchor = attachHost();

    resizeGrid('md');
    stepGrid(0);

    expect(holdAnchor).not.toHaveBeenCalled();
    expect(gridSizeHud.visible).toBe(true);
    expect(gridSizeHud.at).toBeNull();
  });

  it('resizes without a grid on screen', () => {
    resizeGrid('sm');

    expect(browseView.gridSize).toBe('sm');
    expect(gridSizeHud.visible).toBe(true);
  });

  it('places the HUD at a pointer click and centres it for a keyboard click', () => {
    expect(clickOrigin({ detail: 1, clientX: 3, clientY: 4 })).toEqual({
      hud: { x: 3, y: 4 }
    });
    expect(clickOrigin({ detail: 0, clientX: 3, clientY: 4 })).toEqual({});
  });
});
