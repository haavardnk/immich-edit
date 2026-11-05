import { filterPanelHost } from '#lib/commands/grid/filterPanelHost.ts';
import type { Command } from '#lib/commands/types.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';

export const GRID_FILTER_COMMANDS: readonly Command[] = [
  {
    id: 'grid.toggleFilters',
    contexts: ['grid'],
    bind: 'toggleFilters',
    gate: 'outsideFields',
    active: () => browseView.loupeId === null && filterPanelHost.get() !== null,
    run: () => {
      const host = filterPanelHost.get();
      if (browseView.filtersOpen) void host?.close();
      else browseView.setFiltersOpen(true);
    }
  }
];
