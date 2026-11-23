import { gridTargets } from '#lib/browse/gridActions.ts';
import { copyLocationFrom, pasteLocationTo } from '#lib/browse/locationCopy.ts';
import { gridCommand } from '#lib/commands/grid/context.ts';
import type { Command } from '#lib/commands/types.ts';

export const GRID_LOCATION_COMMANDS: readonly Command[] = [
  gridCommand({
    id: 'grid.copyLocation',
    bind: 'copyLocation',
    yields: () => gridTargets().length !== 1,
    run: () => {
      const id = gridTargets()[0];
      if (id) void copyLocationFrom(id);
    }
  }),
  gridCommand({
    id: 'grid.pasteLocation',
    bind: 'pasteLocation',
    yields: () => gridTargets().length === 0,
    run: () => void pasteLocationTo(gridTargets())
  })
];
