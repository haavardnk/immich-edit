import { bulkExportDock } from '#lib/browse/bulk/bulkExportDock.svelte.ts';
import { gridCommand } from '#lib/commands/grid/context.ts';
import type { Command } from '#lib/commands/types.ts';
import { layout } from '#lib/device/layout.svelte.ts';
import { selection } from '#lib/stores/selection.svelte.ts';

export const GRID_EXPORT_COMMANDS: readonly Command[] = [
  gridCommand({
    id: 'grid.openExport',
    bind: 'openExport',
    yields: () => selection.count === 0 || layout.mode === 'touch',
    run: () => bulkExportDock.toggle()
  })
];
