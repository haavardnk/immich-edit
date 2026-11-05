import type { Job } from '#lib/api/jobs.ts';
import { describe, expect, it } from 'vitest';
import {
  etaText,
  jobEtaText,
  jobKindLabel,
  jobKindVerb,
  jobOutcome,
  jobProgress,
  jobsEtaText,
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

  it('describes running jobs for the jobs pill and card', () => {
    expect(
      jobsProgressText([job({ kind: 'export_immich', total: 12, completed: 3, failed: 0 })])
    ).toBe('Exporting 3 of 12');
    expect(jobsProgressText([job({}), job({ id: 'k', total: 6, completed: 1, failed: 0 })])).toBe(
      '2 jobs · 3 of 10'
    );
    expect(jobsProgressText([job({ total: 0, completed: 0, failed: 0 })])).toBe('Zipping');
    expect(
      jobsProgressText([job({ total: 0, completed: 0, failed: 0 }), job({ id: 'k', total: 0 })])
    ).toBe('2 jobs running');
    expect(jobsProgressText([job({ kind: 'mystery', total: 2, completed: 1, failed: 0 })])).toBe(
      'mystery 1 of 2'
    );
  });

  it('rounds an ETA up to whole minutes', () => {
    expect(etaText(0)).toBe('<1 min left');
    expect(etaText(59)).toBe('<1 min left');
    expect(etaText(61)).toBe('2 min left');
    expect(etaText(3540)).toBe('59 min left');
    expect(etaText(3600)).toBe('1 h left');
    expect(etaText(3725)).toBe('1 h 3 min left');
  });

  it('shows the ETA of a single job only when it has one', () => {
    expect(jobEtaText(job({ eta_secs: 90 }))).toBe('2 min left');
    expect(jobEtaText(job({ eta_secs: null, item_ms: 1000 }))).toBeNull();
  });

  it('estimates queued jobs with the running job rate', () => {
    const running = job({ eta_secs: 90, item_ms: 15000 });
    const queued = job({ id: 'k', status: 'pending', total: 10, completed: 0, failed: 0 });
    expect(jobsEtaText([running])).toBe('2 min left');
    expect(jobsEtaText([running, queued])).toBe('4 min left');
    expect(
      jobsEtaText([running, { ...queued, completed: 4, failed: 2 }, { ...queued, id: 'q' }])
    ).toBe('5 min left');
  });

  it('still counts queued jobs once the running job is overdue', () => {
    const overdue = job({ eta_secs: 0, item_ms: 30000 });
    const queued = job({ id: 'k', status: 'pending', total: 4, completed: 0, failed: 0 });
    expect(jobsEtaText([overdue, queued])).toBe('2 min left');
  });

  it('shows no summary ETA until a running job has a rate', () => {
    const queued = job({ id: 'k', status: 'pending', total: 10, completed: 0, failed: 0 });
    expect(jobsEtaText([job({ eta_secs: 90 }), queued])).toBeNull();
    expect(jobsEtaText([job({ eta_secs: null }), queued])).toBeNull();
    expect(jobsEtaText([])).toBeNull();
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
    expect(jobKindVerb('apply_preset')).toBe('Applying preset');
    expect(jobKindVerb('mystery')).toBe('mystery');
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
