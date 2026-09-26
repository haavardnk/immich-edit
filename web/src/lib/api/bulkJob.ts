import type { Job } from '$lib/api/jobs';
import { selection } from '$lib/stores/selection.svelte';
import { jobs } from '$lib/stores/jobs.svelte';
import { toasts } from '$lib/stores/toasts.svelte';

export async function runBulkJob(
  create: (assetIds: string[]) => Promise<Job>,
  error: string,
  assetIds: string[] = [...selection.selected]
): Promise<boolean> {
  if (assetIds.length === 0) return false;
  try {
    jobs.track(await create(assetIds));
    return true;
  } catch (e) {
    toasts.fail(error, e, 6000);
    return false;
  }
}
