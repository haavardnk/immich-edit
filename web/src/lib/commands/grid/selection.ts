import { bulkActions } from '#lib/browse/bulk/bulkActions.svelte.ts';
import { gridCommand, type GridCommandSpec } from '#lib/commands/grid/context.ts';
import { gridHost } from '#lib/commands/grid/gridHost.ts';
import type { Command } from '#lib/commands/types.ts';
import { selection } from '#lib/stores/selection.svelte.ts';

function move(id: string, bind: GridCommandSpec['bind']): Command {
  return gridCommand({
    id,
    bind,
    run: (host, trigger) => {
      if (trigger) host.move(trigger.key, trigger.shiftKey);
    }
  });
}

export const GRID_SELECTION_COMMANDS: readonly Command[] = [
  {
    id: 'grid.cancelSelectAll',
    contexts: ['grid'],
    bind: 'gridClearSelection',
    gate: 'outsideControls',
    active: () => gridHost.get()?.selectingAll() ?? false,
    run: () => gridHost.get()?.cancelSelectAll()
  },
  gridCommand({
    id: 'grid.clearSelection',
    bind: 'gridClearSelection',
    yields: (host) => !selection.selecting && !host.dragging(),
    run: (host) => {
      if (host.dragging()) host.cancelDrag();
      else selection.clear();
    }
  }),
  gridCommand({
    id: 'grid.focusBulkBar',
    bind: 'focusBulkBar',
    yields: () => !selection.active,
    run: () => bulkActions.focusBar()
  }),
  gridCommand({
    id: 'grid.selectAll',
    bind: 'gridSelectAll',
    run: (host) => void host.selectAll()
  }),
  move('grid.move', 'gridMove'),
  move('grid.edge', 'gridEdge'),
  move('grid.page', 'gridPage'),
  move('grid.extend', 'gridExtend')
];
