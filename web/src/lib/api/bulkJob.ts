import type { Job, JobTarget } from '$lib/api/jobs';
import { selection } from '$lib/stores/selection.svelte';
import { jobs } from '$lib/stores/jobs.svelte';
import { toasts } from '$lib/stores/toasts.svelte';

export async function runBulkJob(
  create: (target: JobTarget) => Promise<Job>,
  opts: { success: (count: number) => string; error: string }
): Promise<boolean> {
  const count = selection.targetCount;
  if (count === 0) return false;
  const target = selection.buildTarget();
  try {
    await create(target);
    toasts.push('success', opts.success(count), 4000);
    if (jobs.open) {
      void jobs.load();
    } else {
      jobs.toggle();
    }
    return true;
  } catch (e) {
    toasts.fail(opts.error, e, 6000);
    return false;
  }
}
