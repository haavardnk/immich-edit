import { isRecord } from '#lib/utils/isRecord.ts';

export const SKIP_WAITING = { type: 'skip-waiting' } as const;

export function isSkipWaiting(data: unknown): boolean {
  return isRecord(data) && data.type === SKIP_WAITING.type;
}
