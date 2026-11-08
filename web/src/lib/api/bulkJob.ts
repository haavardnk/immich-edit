import type { Job } from '#lib/api/jobs.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import { jobs } from '#lib/stores/jobs.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';

export async function runBulkJob(
  create: (assetIds: string[]) => Promise<Job>,
  error: string,
  assetIds: string[] = [...selection.selected]
): Promise<Job | null> {
  if (assetIds.length === 0) return null;
  try {
    const job = await create(assetIds);
    jobs.track(job);
    return job;
  } catch (e) {
    toasts.fail(error, e, 6000);
    return null;
  }
}
