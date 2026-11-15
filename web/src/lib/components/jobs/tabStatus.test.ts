import type { Job } from '#lib/api/jobs.ts';
import { describe, expect, it } from 'vitest';
import { tabStatus } from './tabStatus';

const running: Job = {
  id: 'j',
  kind: 'export_immich',
  status: 'running',
  target: null,
  params: null,
  total: 124,
  completed: 50,
  failed: 2,
  cancelled_at: null,
  created_at: '2026-01-01T10:00:00Z',
  updated_at: '2026-01-01T10:00:00Z'
};

describe('tabStatus', () => {
  it('shows the progress of running jobs', () => {
    expect(tabStatus([running], true)).toEqual({
      prefix: '(42%) ',
      badge: { kind: 'progress', fraction: 0.42 }
    });
  });

  it('flags an unseen failure once nothing runs', () => {
    expect(tabStatus([], true)).toEqual({ prefix: '(!) ', badge: { kind: 'alert' } });
  });

  it('leaves the tab plain otherwise', () => {
    expect(tabStatus([], false)).toEqual({ prefix: '', badge: null });
  });
});
