import { gridCommand } from '#lib/commands/grid/context.ts';
import type { Command } from '#lib/commands/types.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';

export const GRID_VIEW_COMMANDS: readonly Command[] = [
  gridCommand({
    id: 'grid.size',
    bind: 'gridSize',
    run: (_, trigger) => {
      if (trigger) browseView.stepGridSize(trigger.key === '-' || trigger.key === '_' ? -1 : 1);
    }
  }),
  gridCommand({
    id: 'grid.tileInfo',
    bind: 'gridTileInfo',
    run: () => browseView.cycleTileInfo()
  })
];
