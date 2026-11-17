import {
  isJobActive,
  JOB_KINDS,
  jobNeedsAttention,
  jobVariants,
  type Job,
  type JobItem
} from '#lib/api/jobs.ts';
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

export function jobItemPreset(job: Job, item: JobItem): string | null {
  return item.result?.preset ?? (jobVariants(job)[item.variant]?.preset || null);
}

export type JobItemState = JobItem['status'] | 'skipped' | 'duplicate' | 'replaced';

export function jobItemState(item: JobItem): JobItemState {
  if (item.status !== 'completed') return item.status;
  const status = item.result?.status;
  if (status === 'skipped' || status === 'duplicate') return status;
  return item.result?.replaced_asset_id ? 'replaced' : 'completed';
}

export function jobOutcome(job: Job): string {
  if (isJobActive(job.status)) return `${job.completed + job.failed} of ${job.total}`;
  if (job.status === 'cancelled') return 'Cancelled';
  if (job.failed > 0) return `${job.failed} of ${job.total} failed`;
  if (job.status === 'failed') return 'Failed';
  const presets = jobVariants(job).length;
  if (presets < 2) return `Done, ${countOf(job.completed, 'photo')}`;
  return `Done, ${countOf(Math.floor(job.completed / presets), 'photo')} × ${countOf(presets, 'preset')}`;
}

export function jobTime(iso: string, now: Date = new Date()): string | null {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return null;
  const today = date.toDateString() === now.toDateString();
  return new Intl.DateTimeFormat(
    undefined,
    today ? { hour: 'numeric', minute: '2-digit' } : { day: 'numeric', month: 'short' }
  ).format(date);
}

export type JobDetailPart = { text: string; danger: boolean };

function detailParts(parts: readonly (string | JobDetailPart | null)[]): JobDetailPart[] {
  return parts
    .filter((part) => part !== null)
    .map((part) => (typeof part === 'string' ? { text: part, danger: false } : part));
}

export function jobDetail(job: Job, now: Date = new Date()): JobDetailPart[] {
  const done = `${job.completed + job.failed} of ${job.total}`;
  if (isJobActive(job.status)) {
    const presets = jobVariants(job).length;
    const started = jobTime(job.created_at, now);
    return detailParts([
      done,
      presets > 1 ? countOf(presets, 'preset') : null,
      jobEtaText(job) ?? (started ? `started ${started}` : null),
      job.failed > 0 ? { text: `${job.failed} failed`, danger: true } : null
    ]);
  }
  const ended = jobTime(job.updated_at, now);
  if (job.status === 'cancelled') return detailParts(['Cancelled', `${done} done`, ended]);
  return detailParts([{ text: jobOutcome(job), danger: jobNeedsAttention(job) }, ended]);
}

export function jobErrorsText(job: Job, items: readonly JobItem[]): string {
  const lines = items
    .filter((item) => item.status === 'failed')
    .map((item) => {
      const preset = jobItemPreset(job, item);
      const name = item.result?.filename ?? item.asset_id;
      return `${name}${preset ? ` (${preset})` : ''}: ${item.error ?? 'failed'}`;
    });
  return [`${jobKindLabel(job.kind)} ${job.id}: ${jobOutcome(job)}`, ...lines].join('\n');
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
