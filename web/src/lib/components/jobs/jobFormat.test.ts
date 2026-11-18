import type { Job } from '#lib/api/jobs.ts';
import { describe, expect, it } from 'vitest';
import { jobKindLabel, jobOutcome, jobProgress, jobsSummary } from './jobFormat';

function job(patch: Partial<Job>): Job {
  return {
    id: 'j',
    kind: 'download_zip',
    status: 'running',
    target: null,
    params: null,
    total: 4,
    completed: 1,
    failed: 1,
    cancelled_at: null,
    created_at: '2026-01-01T10:00:00Z',
    updated_at: '2026-01-01T10:00:00Z',
    ...patch
  };
}

describe('jobFormat', () => {
  it('summarises the active job count', () => {
    expect(jobsSummary(0)).toBe('No active jobs');
    expect(jobsSummary(2)).toBe('2 active');
  });

  it('labels known kinds and passes unknown kinds through', () => {
    expect(jobKindLabel('download_zip')).toBe('Download ZIP');
    expect(jobKindLabel('mystery')).toBe('mystery');
  });

  it('reports progress and outcome', () => {
    expect(jobProgress(job({}))).toBe(50);
    expect(jobProgress(job({ total: 0 }))).toBe(0);
    expect(jobOutcome(job({}))).toBe('2 of 4');
    expect(jobOutcome(job({ status: 'completed', failed: 0, completed: 4 }))).toBe(
      'Done, 4 photos'
    );
  });
});
