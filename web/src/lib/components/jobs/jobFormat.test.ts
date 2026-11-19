import type { Job, JobItem } from '#lib/api/jobs.ts';
import { describe, expect, it } from 'vitest';
import {
  etaText,
  jobEtaText,
  jobKindLabel,
  jobKindVerb,
  jobItemPreset,
  jobItemState,
  jobDetail,
  jobErrorsText,
  jobOutcome,
  jobProgress,
  jobTime,
  jobsEtaText,
  jobsProgressFraction,
  jobsProgressText
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

  it('counts photos, not files, in a multi-preset job', () => {
    const variants = { variants: [{ preset: 'Web' }, { preset: 'Print' }] };
    expect(
      jobOutcome(job({ status: 'completed', failed: 0, completed: 4, params: variants }))
    ).toBe('Done, 2 photos × 2 presets');
    expect(jobOutcome(job({ status: 'completed', failed: 1, params: variants }))).toBe(
      '1 of 4 failed'
    );
    expect(
      jobOutcome(
        job({ status: 'completed', failed: 0, completed: 3, total: 3, params: { variants: [{}] } })
      )
    ).toBe('Done, 3 photos');
  });

  it('names the preset of a job item', () => {
    const variants = job({ params: { variants: [{ preset: 'Web' }, { preset: 'Print' }] } });
    const item = { variant: 1, result: null } as JobItem;
    expect(jobItemPreset(variants, item)).toBe('Print');
    expect(jobItemPreset(variants, { ...item, result: { preset: 'Print 2' } })).toBe('Print 2');
    expect(jobItemPreset(job({}), { ...item, variant: 0 })).toBeNull();
  });

  it('tells skipped and updated uploads apart from plain ones', () => {
    const done = { status: 'completed', result: { filename: 'a.jpg' } } as JobItem;
    expect(jobItemState(done)).toBe('completed');
    expect(jobItemState({ ...done, result: { status: 'skipped' } })).toBe('skipped');
    expect(jobItemState({ ...done, result: { status: 'duplicate' } })).toBe('duplicate');
    expect(jobItemState({ ...done, result: { replaced_asset_id: 'old' } })).toBe('replaced');
    expect(jobItemState({ ...done, status: 'failed' })).toBe('failed');
  });

  const now = new Date('2026-01-01T10:30:00Z');
  const clock = (iso: string): string =>
    new Intl.DateTimeFormat(undefined, { hour: 'numeric', minute: '2-digit' }).format(
      new Date(iso)
    );

  it('shows the time of day for today and the date before that', () => {
    expect(jobTime('2026-01-01T10:00:00Z', now)).toBe(clock('2026-01-01T10:00:00Z'));
    expect(jobTime('2025-12-24T10:00:00Z', now)).toBe(
      new Intl.DateTimeFormat(undefined, { day: 'numeric', month: 'short' }).format(
        new Date('2025-12-24T10:00:00Z')
      )
    );
    expect(jobTime('soon', now)).toBeNull();
  });

  it('describes a running job by count, presets and time', () => {
    const variants = { variants: [{ preset: 'Web' }, { preset: 'Print' }] };
    expect(jobDetail(job({ params: variants }), now)).toEqual([
      { text: '2 of 4', danger: false },
      { text: '2 presets', danger: false },
      { text: `started ${clock('2026-01-01T10:00:00Z')}`, danger: false },
      { text: '1 failed', danger: true }
    ]);
    expect(jobDetail(job({ failed: 0, eta_secs: 90 }), now).map((part) => part.text)).toEqual([
      '1 of 4',
      '2 min left'
    ]);
  });

  it('describes a finished job by outcome and when it ended', () => {
    const ended = clock('2026-01-01T10:00:00Z');
    expect(jobDetail(job({ status: 'completed', completed: 4, failed: 0 }), now)).toEqual([
      { text: 'Done, 4 photos', danger: false },
      { text: ended, danger: false }
    ]);
    expect(jobDetail(job({ status: 'completed', completed: 2, failed: 2 }), now)).toEqual([
      { text: '2 of 4 failed', danger: true },
      { text: ended, danger: false }
    ]);
    expect(jobDetail(job({ status: 'cancelled', failed: 0 }), now).map((p) => p.text)).toEqual([
      'Cancelled',
      '1 of 4 done',
      ended
    ]);
  });

  it('lists failed items with their preset for a bug report', () => {
    const variants = job({
      id: 'job-7',
      status: 'completed',
      completed: 1,
      failed: 2,
      total: 3,
      params: { variants: [{ preset: 'Web' }, { preset: 'Print' }] }
    });
    const item = (patch: Partial<JobItem>): JobItem =>
      ({ asset_id: 'asset-1', variant: 0, status: 'failed', result: null, ...patch }) as JobItem;
    expect(
      jobErrorsText(variants, [
        item({ error: 'Upload timed out', result: { filename: 'a_web.jpg' } }),
        item({ status: 'completed', error: null }),
        item({ variant: 1, error: null })
      ])
    ).toBe(
      'Download ZIP job-7: 2 of 3 failed\na_web.jpg (Web): Upload timed out\nasset-1 (Print): failed'
    );
  });
});
