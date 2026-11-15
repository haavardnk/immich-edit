import type { Job, JobItem } from '#lib/api/jobs.ts';
import { jobErrorsText } from '#lib/components/jobs/jobFormat.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';

export async function copyJobErrors(job: Job, items: readonly JobItem[]): Promise<void> {
  try {
    await navigator.clipboard.writeText(jobErrorsText(job, items));
    toasts.push('success', 'Copied the errors');
  } catch (error) {
    toasts.fail('Could not copy the errors', error);
  }
}
