import { isJobActive, type Job } from '$lib/api/jobs';

const KIND_LABELS: Record<string, string> = {
  export_immich: 'Export to Immich',
  download_zip: 'Download ZIP',
  apply_preset: 'Apply Preset',
  paste_edits: 'Paste Edits',
  reset_edits: 'Reset Edits'
};

export function jobKindLabel(kind: string): string {
  return KIND_LABELS[kind] ?? kind;
}

export function jobProgress(job: Job): number {
  if (!job.total) return 0;
  return Math.round(((job.completed + job.failed) / job.total) * 100);
}

export function jobOutcome(job: Job): string {
  if (isJobActive(job.status)) return `${job.completed + job.failed} of ${job.total}`;
  if (job.status === 'cancelled') return 'Cancelled';
  if (job.failed > 0) return `${job.failed} of ${job.total} failed`;
  if (job.status === 'failed') return 'Failed';
  return `Done, ${job.completed} photo${job.completed === 1 ? '' : 's'}`;
}
