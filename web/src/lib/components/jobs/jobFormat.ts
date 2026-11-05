import { isJobActive, JOB_KINDS, type Job } from '#lib/api/jobs.ts';
import { countOf } from '#lib/utils/plural.ts';

const KIND_LABELS: Record<string, string> = {
  [JOB_KINDS.exportImmich]: 'Export to Immich',
  [JOB_KINDS.downloadZip]: 'Download ZIP',
  [JOB_KINDS.applyPreset]: 'Apply Preset',
  [JOB_KINDS.pasteEdits]: 'Paste Edits',
  [JOB_KINDS.resetEdits]: 'Reset Edits'
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
  return `Done, ${countOf(job.completed, 'photo')}`;
}

export function jobsSummary(activeCount: number): string {
  return activeCount > 0 ? `${activeCount} active` : 'No active jobs';
}

export function jobsProgressText(active: readonly Job[]): string {
  const done = active.reduce((sum, job) => sum + job.completed + job.failed, 0);
  const total = active.reduce((sum, job) => sum + job.total, 0);
  const [only] = active;
  const subject =
    only && active.length === 1
      ? jobKindLabel(only.kind)
      : `${countOf(active.length, 'job')} running`;
  return total > 0 ? `${subject} · ${done} of ${total}` : subject;
}
