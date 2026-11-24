import { clockOfDay } from '#lib/geotag/clockText.ts';
import type { PhotoMatch } from '#lib/geotag/match.ts';
import type { CameraTime } from '#lib/geotag/photoTime.ts';

export type MatchTone = 'ok' | 'out' | 'kept';

const MINUTE = 60_000;

function minutes(ms: number): string {
  const value = Math.round(ms / MINUTE);
  return value < 120 ? `${value} min` : `${Math.round(value / 60)} h`;
}

export function matchStatus(match: PhotoMatch): { text: string; tone: MatchTone } {
  const outcome = match.outcome;
  if (outcome.kind === 'placed') return { text: 'On the track', tone: 'ok' };
  if (outcome.kind === 'kept') return { text: 'Keeps its location', tone: 'kept' };
  if (outcome.kind === 'untimed') return { text: 'No capture time', tone: 'out' };
  if (outcome.reason === 'before')
    return { text: `${minutes(outcome.gapMs)} before start`, tone: 'out' };
  if (outcome.reason === 'after')
    return { text: `${minutes(outcome.gapMs)} after end`, tone: 'out' };
  return { text: `${minutes(outcome.gapMs)} from the track`, tone: 'out' };
}

export function matchTimes(
  base: CameraTime | null | undefined,
  match: PhotoMatch,
  zone: string
): string | null {
  if (!base || match.time === null) return null;
  return `${clockOfDay(base.instant, zone)} → ${clockOfDay(match.time, zone)}`;
}

export const MATCH_TONES = {
  ok: 'bg-primary/10 text-primary',
  out: 'bg-warning-500/10 text-warning-800',
  kept: 'bg-ghost text-dark/65'
} as const satisfies Record<MatchTone, string>;
