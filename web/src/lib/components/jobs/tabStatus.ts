import type { Job } from '#lib/api/jobs.ts';
import { jobsProgressFraction } from '#lib/components/jobs/jobFormat.ts';
import type { FaviconBadge } from '#lib/theme/favicon.ts';

export interface TabStatus {
  prefix: string;
  badge: FaviconBadge | null;
}

export function tabStatus(active: readonly Job[], attention: boolean): TabStatus {
  if (active.length > 0) {
    const percent = Math.round(jobsProgressFraction(active) * 100);
    return { prefix: `(${percent}%) `, badge: { kind: 'progress', fraction: percent / 100 } };
  }
  if (attention) return { prefix: '(!) ', badge: { kind: 'alert' } };
  return { prefix: '', badge: null };
}
