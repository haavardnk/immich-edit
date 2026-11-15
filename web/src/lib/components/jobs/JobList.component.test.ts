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
import JobsPanel from './JobsPanel.svelte';
import JobsSheet from './JobsSheet.svelte';

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
  position: 1,
  variant: 0,
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
  it('exposes the same controls standalone and in the jobs panel', async () => {
    const found = await layoutControls([
      layout('body', JobList, {}),
      layout('panel', JobsPanel, {}, { exclude: [button('Close jobs'), button('Clear finished')] })
    ]);
    expect(found.body).toEqual([
      button('Cancel job'),
      button('Copy errors'),
      { ...button('Download ZIP details'), expanded: true },
      { ...button('Export to Immich details'), expanded: false },
      { role: 'link', name: 'Download ZIP' }
    ]);
    expect(found.panel).toEqual(found.body);
  });

  it('exposes the list and its actions together in the jobs panel', async () => {
    const found = await layoutControls([
      layout('list', JobList, {}),
      layout('actions', JobListActions, {}),
      layout('panel', JobsPanel, {}, { exclude: [button('Close jobs')] })
    ]);
    expect(found.actions).toEqual([button('Clear finished')]);
    expect(found.panel).toEqual(mergeControls(found.list, found.actions));
  });

  it('exposes the grouped list and its actions in the jobs sheet', async () => {
    const found = await layoutControls([
      layout('list', JobList, { grouped: true }),
      layout('actions', JobListActions, {}),
      layout('sheet', JobsSheet, {}, { exclude: [button('Close jobs')] })
    ]);
    expect(found.sheet).toEqual(mergeControls(found.list, found.actions));
  });

  it('shows the ETA of a running job in its row', async () => {
    jobs.jobs = [{ ...job('run', 'export_immich', 'running'), eta_secs: 30 }];
    const target = document.createElement('div');
    document.body.appendChild(target);
    const destroy = layout('list', JobList, {}).render(target);
    expect(target.textContent).toMatch(/1 of 2\s*·\s*<1 min left/);
    await destroy();
    target.remove();
  });

  it('lists running jobs before finished ones only when grouped', async () => {
    jobs.jobs = [job('zip', 'download_zip', 'completed'), job('run', 'export_immich', 'running')];
    const headings = async (grouped: boolean): Promise<string[]> => {
      const target = document.createElement('div');
      document.body.appendChild(target);
      const destroy = layout('list', JobList, { grouped }).render(target);
      const found = [...target.querySelectorAll('h3, [aria-label$=" details"]')].map(
        (element) => element.textContent?.trim() || (element.getAttribute('aria-label') ?? '')
      );
      await destroy();
      target.remove();
      return found;
    };
    expect(await headings(false)).toEqual(['Download ZIP details', 'Export to Immich details']);
    expect(await headings(true)).toEqual([
      'Running',
      'Export to Immich details',
      'Finished',
      'Download ZIP details'
    ]);
  });

  it('names the preset of each item in a multi-preset job', async () => {
    jobs.jobs = [
      {
        ...job('zip', 'download_zip', 'completed'),
        params: { variants: [{ preset: 'Web' }, { preset: 'Print' }] }
      }
    ];
    jobs.items = {
      zip: [
        { ...item, status: 'completed', result: { filename: 'a_web.jpg', preset: 'Web' } },
        { ...item, id: 'i2', variant: 1, status: 'pending', error: null, result: null }
      ]
    };
    const target = document.createElement('div');
    document.body.appendChild(target);
    const destroy = layout('list', JobList, {}).render(target);
    expect(target.textContent).toMatch(/a_web\.jpg\s*Web\s*completed/);
    expect(target.textContent).toMatch(/asset-00\s*Print\s*pending/);
    await destroy();
    target.remove();
  });
});
