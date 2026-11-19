import { beforeEach, describe, expect, it } from 'vitest';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { jobs } from '#lib/stores/jobs.svelte.ts';
import { bulkExportDock } from './bulkExportDock.svelte';

beforeEach(() => {
  bulkExportDock.close();
  jobs.open = false;
  browseView.setFiltersOpen(false);
});

describe('bulkExportDock', () => {
  it('takes the side slot from Jobs and Filters when it opens', () => {
    jobs.open = true;
    browseView.setFiltersOpen(true);
    bulkExportDock.show();
    expect(bulkExportDock.open).toBe(true);
    expect(jobs.open).toBe(false);
    expect(browseView.filtersOpen).toBe(false);
  });

  it('toggles open and closed', () => {
    bulkExportDock.toggle();
    expect(bulkExportDock.open).toBe(true);
    bulkExportDock.toggle();
    expect(bulkExportDock.open).toBe(false);
  });
});
