import { splitJobs, type Job } from '#lib/api/jobs.ts';
import { describe, expect, it } from 'vitest';
import { jobSections } from './jobSections';

function job(id: string, status: Job['status']): Job {
  return {
    id,
    kind: 'export_immich',
    status,
    target: null,
    params: null,
    total: 2,
    completed: 1,
    failed: 0,
    cancelled_at: null,
    created_at: '2026-01-01T10:00:00Z',
    updated_at: '2026-01-01T10:00:00Z'
  };
}

const JOBS = [
  job('done', 'completed'),
  job('run', 'running'),
  job('failed', 'failed'),
  job('queued', 'pending'),
  job('stopped', 'cancelled')
];

const ids = (list: readonly Job[]) => list.map((entry) => entry.id);

describe('splitJobs', () => {
  it('splits running from finished jobs and keeps the newest-first order', () => {
    const { running, finished } = splitJobs(JOBS);
    expect(ids(running)).toEqual(['run', 'queued']);
    expect(ids(finished)).toEqual(['done', 'failed', 'stopped']);
  });
});

describe('jobSections', () => {
  it('keeps one unlabelled section in the original order when not grouped', () => {
    const sections = jobSections(JOBS, false);
    expect(sections.map((section) => section.label)).toEqual([null]);
    expect(ids(sections[0]?.jobs ?? [])).toEqual(ids(JOBS));
  });

  it('puts running jobs first and finished ones after when grouped', () => {
    const sections = jobSections(JOBS, true);
    expect(sections.map((section) => [section.label, ids(section.jobs)])).toEqual([
      ['In progress', ['run', 'queued']],
      ['Finished', ['done', 'failed', 'stopped']]
    ]);
  });

  it('drops empty sections', () => {
    expect(jobSections([], false)).toEqual([]);
    expect(jobSections([], true)).toEqual([]);
    expect(jobSections([job('done', 'completed')], true).map((section) => section.label)).toEqual([
      'Finished'
    ]);
  });
});
