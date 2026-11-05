import { afterEach, describe, expect, it, vi } from 'vitest';
import { signOut } from '#lib/auth/account.ts';
import { signOutFlow } from '#lib/auth/signOutFlow.svelte.ts';
import { jobs } from '#lib/stores/jobs.svelte.ts';
import type { Job } from '#lib/api/jobs.ts';

vi.mock('#lib/auth/account.ts', () => ({ signOut: vi.fn().mockResolvedValue(undefined) }));

function running(kind: string): Job {
  return {
    id: kind,
    kind,
    status: 'running',
    target: null,
    params: null,
    total: 1,
    completed: 0,
    failed: 0,
    cancelled_at: null,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z'
  };
}

afterEach(() => {
  jobs.jobs = [];
  signOutFlow.cancel();
  vi.mocked(signOut).mockClear();
});

describe('signOutFlow', () => {
  it('signs out at once when no export or upload runs', async () => {
    jobs.jobs = [running('apply_preset')];

    signOutFlow.request();
    await vi.waitFor(() => expect(signOut).toHaveBeenCalledOnce());
    expect(signOutFlow.confirming).toBe(false);
  });

  it('asks first while a transfer runs and signs out on confirm', async () => {
    jobs.jobs = [running('download_zip')];

    signOutFlow.request();
    expect(signOutFlow.confirming).toBe(true);
    expect(signOut).not.toHaveBeenCalled();

    await signOutFlow.confirm();
    expect(signOutFlow.confirming).toBe(false);
    expect(signOut).toHaveBeenCalledOnce();
  });

  it('stays signed in when the confirm is cancelled', () => {
    jobs.jobs = [running('export_immich')];

    signOutFlow.request();
    signOutFlow.cancel();
    expect(signOutFlow.confirming).toBe(false);
    expect(signOut).not.toHaveBeenCalled();
  });
});
