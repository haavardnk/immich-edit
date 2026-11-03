import { gridCommand, type GridCommandSpec } from '#lib/commands/grid/context.ts';
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
  gridCommand({
    id: 'grid.clearSelection',
    bind: 'gridClearSelection',
    yields: () => !selection.active,
    run: () => selection.clear()
  }),
  gridCommand({
    id: 'grid.focusBulkBar',
    bind: 'focusBulkBar',
    yields: () => !selection.active,
    run: (host) => host.focusBulkBar()
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
