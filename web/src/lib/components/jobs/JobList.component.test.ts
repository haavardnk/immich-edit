import type { Job, JobItem } from '#lib/api/jobs.ts';
import { jobs } from '#lib/stores/jobs.svelte.ts';
import {
  layout,
  layoutControls,
  mergeControls,
  type ControlRef
} from '#lib/testing/layoutParity.ts';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import JobList from './JobList.svelte';
import JobListActions from './JobListActions.svelte';
import JobsDrawer from './JobsDrawer.svelte';

const button = (name: string): ControlRef => ({ role: 'button', name });

function job(id: string, kind: string, status: Job['status']): Job {
  return {
    id,
    kind,
    status,
    target: null,
    params: null,
    total: 2,
    completed: status === 'completed' ? 2 : 1,
    failed: 0,
    cancelled_at: null,
    created_at: '2026-01-01T10:00:00Z',
    updated_at: '2026-01-01T10:00:00Z'
  };
}

const item: JobItem = {
  id: 'i1',
  job_id: 'zip',
  asset_id: 'asset-0001',
  status: 'failed',
  error: 'Render failed',
  result: { warnings: ['Lens profile missing'] },
  idempotency_key: null,
  attempts: 1,
  created_at: '2026-01-01T10:00:00Z',
  updated_at: '2026-01-01T10:00:00Z'
};

beforeEach(() => {
  jobs.jobs = [job('run', 'export_immich', 'running'), job('zip', 'download_zip', 'completed')];
  jobs.items = { zip: [item] };
  jobs.expanded = 'zip';
  jobs.open = true;
});

afterEach(() => {
  jobs.open = false;
  jobs.expanded = null;
  jobs.items = {};
  jobs.jobs = [];
});

describe('JobList layout parity', () => {
  it('exposes the same controls standalone and in the jobs drawer', async () => {
    const found = await layoutControls([
      layout('body', JobList, {}),
      layout(
        'drawer',
        JobsDrawer,
        {},
        { exclude: [button('Close jobs'), button('Clear finished')] }
      )
    ]);
    expect(found.body).toEqual([
      button('Cancel job'),
      button('Toggle details'),
      button('Toggle details'),
      { role: 'link', name: 'Download ZIP' }
    ]);
    expect(found.drawer).toEqual(found.body);
  });

  it('exposes the list and its actions together in the jobs drawer', async () => {
    const found = await layoutControls([
      layout('list', JobList, {}),
      layout('actions', JobListActions, {}),
      layout('drawer', JobsDrawer, {}, { exclude: [button('Close jobs')] })
    ]);
    expect(found.actions).toEqual([button('Clear finished')]);
    expect(found.drawer).toEqual(mergeControls(found.list, found.actions));
  });
});
