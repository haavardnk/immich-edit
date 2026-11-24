import { DateTime } from 'luxon';
import type { LocalSpan } from '#lib/types/search.ts';

export type DateBoundKey = 'takenAfter' | 'takenBefore' | 'addedAfter' | 'addedBefore';

const BOUND = /^(\d{4}-\d{2}-\d{2})(?:T((?:[01]\d|2[0-3]):[0-5]\d))?$/;
const NAIVE = "yyyy-MM-dd'T'HH:mm:ss";

export function boundAt(bound: string): DateTime {
  return DateTime.fromISO(bound, { zone: 'utc' });
}

export function readBound(value: string | null): string {
  if (!value || !BOUND.test(value)) return '';
  return boundAt(value).isValid ? value : '';
}

export function boundDay(bound: string): string {
  return BOUND.exec(bound)?.[1] ?? '';
}

export function boundTime(bound: string): string {
  return BOUND.exec(bound)?.[2] ?? '';
}

export function joinBound(day: string, time: string): string {
  if (!day) return '';
  return time ? `${day}T${time}` : day;
}

export function boundStart(bound: string): string {
  return boundAt(bound).toFormat(NAIVE);
}

export function boundEnd(bound: string): string {
  const step = boundTime(bound) ? { minutes: 1 } : { days: 1 };
  return boundAt(bound).plus(step).toFormat(NAIVE);
}

export function localSpan(after: string, before: string): LocalSpan | undefined {
  if (!after && !before) return undefined;
  return {
    ...(after ? { from: boundStart(after) } : {}),
    ...(before ? { until: boundEnd(before) } : {})
  };
}

export function holdsWallClock(span: LocalSpan, wallClock: string): boolean {
  if (!wallClock) return false;
  if (span.from && wallClock < span.from) return false;
  return !span.until || wallClock < span.until;
}

export function startInstant(bound: string): string {
  return new Date(boundStart(bound)).toISOString();
}

export function endInstant(bound: string): string {
  return new Date(new Date(boundEnd(bound)).getTime() - 1).toISOString();
}
