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

const KIND_VERBS: Record<string, string> = {
  [JOB_KINDS.exportImmich]: 'Exporting',
  [JOB_KINDS.downloadZip]: 'Zipping',
  [JOB_KINDS.applyPreset]: 'Applying preset',
  [JOB_KINDS.pasteEdits]: 'Pasting edits',
  [JOB_KINDS.resetEdits]: 'Resetting edits'
};

export function jobKindVerb(kind: string): string {
  return KIND_VERBS[kind] ?? jobKindLabel(kind);
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

function activeTotals(active: readonly Job[]): { done: number; total: number } {
  return {
    done: active.reduce((sum, job) => sum + job.completed + job.failed, 0),
    total: active.reduce((sum, job) => sum + job.total, 0)
  };
}

export function jobsProgressFraction(active: readonly Job[]): number {
  const { done, total } = activeTotals(active);
  return total > 0 ? Math.min(1, done / total) : 0;
}

export function etaText(seconds: number): string {
  if (seconds < 60) return '<1 min left';
  const minutes = Math.ceil(seconds / 60);
  if (minutes < 60) return `${minutes} min left`;
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  return rest > 0 ? `${hours} h ${rest} min left` : `${hours} h left`;
}

export function jobEtaText(job: Job): string | null {
  return typeof job.eta_secs === 'number' ? etaText(job.eta_secs) : null;
}

function remainingSeconds(job: Job, itemMs: number): number {
  if (typeof job.eta_secs === 'number') return job.eta_secs;
  return (Math.max(0, job.total - job.completed - job.failed) * itemMs) / 1000;
}

export function jobsEtaText(active: readonly Job[]): string | null {
  const itemMs = active.find((job) => typeof job.item_ms === 'number')?.item_ms;
  if (typeof itemMs !== 'number') return null;
  return etaText(active.reduce((sum, job) => sum + remainingSeconds(job, itemMs), 0));
}

export function jobsProgressText(active: readonly Job[]): string {
  const { done, total } = activeTotals(active);
  const [only] = active;
  if (only && active.length === 1) {
    const verb = jobKindVerb(only.kind);
    return total > 0 ? `${verb} ${done} of ${total}` : verb;
  }
  const jobCount = countOf(active.length, 'job');
  return total > 0 ? `${jobCount} · ${done} of ${total}` : `${jobCount} running`;
}
