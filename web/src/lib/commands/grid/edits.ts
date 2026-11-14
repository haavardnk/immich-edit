import { copyEditsFrom, pasteEditsTo, pasteOnlyTo } from '#lib/browse/copyPaste.ts';
import { gridTargets } from '#lib/browse/gridActions.ts';
import { gridCommand } from '#lib/commands/grid/context.ts';
import type { Command } from '#lib/commands/types.ts';
import { virtualCopies } from '#lib/stores/virtualCopies.svelte.ts';
import { hasTextSelection } from '#lib/shortcuts/targets.ts';

function soleTarget(): string | null {
  const ids = gridTargets();
  return ids.length === 1 ? (ids[0] ?? null) : null;
}

export const GRID_EDIT_COMMANDS: readonly Command[] = [
  gridCommand({
    id: 'grid.copyEdits',
    bind: 'copyEdits',
    yields: () => !soleTarget() || hasTextSelection(),
    run: () => {
      const id = soleTarget();
      if (id) void copyEditsFrom(id);
    }
  }),
  gridCommand({
    id: 'grid.pasteEdits',
    bind: 'pasteEdits',
    yields: () => gridTargets().length === 0 || hasTextSelection(),
    run: () => void pasteEditsTo(gridTargets())
  }),
  gridCommand({
    id: 'grid.pasteOnly',
    bind: 'pasteOnly',
    yields: () => gridTargets().length === 0 || hasTextSelection(),
    run: () => pasteOnlyTo(gridTargets())
  }),
  gridCommand({
    id: 'grid.virtualCopy',
    bind: 'createVirtualCopy',
    yields: (_, trigger) => trigger.repeat || gridTargets().length === 0,
    run: () => void virtualCopies.create(gridTargets())
  })
];
