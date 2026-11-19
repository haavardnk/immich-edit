import { browseView } from '#lib/stores/browseView.svelte.ts';
import { jobs } from '#lib/stores/jobs.svelte.ts';

export const BULK_EXPORT_BUTTON = 'bulk-export-button';

export const BULK_EXPORT_PANEL = 'bulk-export-panel';

class BulkExportDock {
  open = $state(false);

  show = (): void => {
    if (jobs.open) jobs.close();
    if (browseView.filtersOpen) browseView.setFiltersOpen(false);
    this.open = true;
  };

  close = (): void => {
    this.open = false;
  };

  toggle = (): void => {
    if (this.open) this.close();
    else this.show();
  };
}

export const bulkExportDock = new BulkExportDock();
