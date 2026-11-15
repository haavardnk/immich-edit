import type { Job } from '#lib/api/jobs.ts';
import { describe, expect, it } from 'vitest';
import { progressed, supersedes } from './transitions';

function job(status: Job['status'], updated_at: string): Job {
  return {
    id: 'j',
    kind: 'export_immich',
    status,
    target: null,
    params: null,
    total: 2,
    completed: 1,
    failed: 1,
    cancelled_at: null,
    created_at: '2026-01-01T10:00:00Z',
    updated_at
  };
}

describe('supersedes', () => {
  const at = (seconds: number): string => `2026-01-01T10:00:${String(seconds).padStart(2, '0')}Z`;

  it('takes the first record and every later one', () => {
    expect(supersedes(undefined, job('running', at(0)))).toBe(true);
    expect(supersedes(job('running', at(0)), job('running', at(1)))).toBe(true);
    expect(supersedes(job('running', at(1)), job('completed', at(1)))).toBe(true);
  });

  it('drops a record older than the one it would replace', () => {
    expect(supersedes(job('pending', at(5)), job('completed', at(2)))).toBe(false);
    expect(supersedes(job('running', at(5)), job('running', at(2)))).toBe(false);
  });

  it('reopens a finished job only for a newer record', () => {
    expect(supersedes(job('completed', at(2)), job('running', at(2)))).toBe(false);
    expect(supersedes(job('completed', at(2)), job('pending', at(3)))).toBe(true);
  });

  it('reads nanosecond timestamps', () => {
    expect(
      supersedes(
        job('completed', '2026-01-01T10:00:00.100000000+00:00'),
        job('pending', '2026-01-01T10:00:00.200000000+00:00')
      )
    ).toBe(true);
  });
});

describe('progressed', () => {
  const at = '2026-01-01T10:00:00Z';

  it('sees a new status or count', () => {
    expect(progressed(job('running', at), job('completed', at))).toBe(true);
    expect(progressed(job('running', at), { ...job('running', at), completed: 2 })).toBe(true);
    expect(progressed(job('running', at), { ...job('running', at), failed: 2 })).toBe(true);
  });

  it('ignores the first record and a repeat', () => {
    expect(progressed(undefined, job('running', at))).toBe(false);
    expect(progressed(job('running', at), job('running', '2026-01-01T10:00:09Z'))).toBe(false);
  });
});
