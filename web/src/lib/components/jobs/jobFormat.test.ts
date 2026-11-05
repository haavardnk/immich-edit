import type { Job } from '#lib/api/jobs.ts';
import { describe, expect, it } from 'vitest';
import {
  jobKindLabel,
  jobOutcome,
  jobProgress,
  jobsProgressFraction,
  jobsProgressText,
  jobsSummary
} from './jobFormat';

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

  it('describes running jobs for the jobs pill', () => {
    expect(
      jobsProgressText([job({ kind: 'export_immich', total: 12, completed: 3, failed: 0 })])
    ).toBe('Export to Immich · 3 of 12');
    expect(jobsProgressText([job({}), job({ id: 'k', total: 6, completed: 1, failed: 0 })])).toBe(
      '2 jobs running · 3 of 10'
    );
    expect(jobsProgressText([job({ total: 0, completed: 0, failed: 0 })])).toBe('Download ZIP');
  });

  it('reports the combined fraction of active jobs', () => {
    expect(
      jobsProgressFraction([job({}), job({ id: 'k', total: 6, completed: 1, failed: 0 })])
    ).toBe(0.3);
    expect(jobsProgressFraction([job({ total: 0, completed: 0, failed: 0 })])).toBe(0);
    expect(jobsProgressFraction([job({ total: 2, completed: 3, failed: 0 })])).toBe(1);
    expect(jobsProgressFraction([])).toBe(0);
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
