import { isJobActive, type Job } from '#lib/api/jobs.ts';

function updatedAt(job: Job): number | null {
  const time = Date.parse(job.updated_at);
  return Number.isNaN(time) ? null : time;
}

function order(previous: Job, next: Job): number {
  const before = updatedAt(previous);
  const after = updatedAt(next);
  return before === null || after === null ? 0 : Math.sign(after - before);
}

export function revives(previous: Job | undefined, next: Job): boolean {
  return previous !== undefined && !isJobActive(previous.status) && isJobActive(next.status);
}

export function endedNow(previous: Job | undefined, next: Job): boolean {
  return previous !== undefined && isJobActive(previous.status) && !isJobActive(next.status);
}

export function supersedes(previous: Job | undefined, next: Job): boolean {
  if (previous === undefined) return true;
  const newer = order(previous, next);
  if (newer < 0) return false;
  return !revives(previous, next) || newer > 0;
}

export function progressed(previous: Job | undefined, next: Job): boolean {
  return (
    previous !== undefined &&
    (previous.status !== next.status ||
      previous.completed !== next.completed ||
      previous.failed !== next.failed)
  );
}
