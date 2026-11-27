import type { PhotoMatch } from '#lib/geotag/match.ts';
import { referenceOf } from '#lib/geotag/references.ts';
import { distanceText } from '#lib/geotag/spots.ts';
import type { Located } from '#lib/geotag/trackLog.ts';
import { clockOfDay } from '#lib/metadata/clockText.ts';
import type { CameraTime } from '#lib/metadata/photoTime.ts';

export type MatchTone = 'ok' | 'out' | 'kept';

const MINUTE = 60_000;

function minutes(ms: number): string {
  const value = Math.round(ms / MINUTE);
  return value < 120 ? `${value} min` : `${Math.round(value / 60)} h`;
}

function outsideText(outcome: Extract<Located, { kind: 'outside' }>): string {
  if (outcome.reason === 'before') return `${minutes(outcome.gapMs)} before start`;
  if (outcome.reason === 'after') return `${minutes(outcome.gapMs)} after end`;
  return `${minutes(outcome.gapMs)} from the track`;
}

export function matchStatus(match: PhotoMatch): { text: string; tone: MatchTone } {
  const outcome = match.outcome;
  const reference = referenceOf(outcome);
  if (outcome.kind === 'placed') {
    return reference?.moved && reference.metres !== null
      ? { text: `Moves ${distanceText(reference.metres)}`, tone: 'ok' }
      : { text: 'On the track', tone: 'ok' };
  }
  if (outcome.kind === 'kept') {
    const where =
      outcome.track.kind === 'outside'
        ? outsideText(outcome.track)
        : `${distanceText(reference?.metres ?? 0)} from the track`;
    return { text: `Has a location · ${where}`, tone: 'kept' };
  }
  if (outcome.kind === 'untimed') return { text: 'No capture time', tone: 'out' };
  return { text: outsideText(outcome), tone: 'out' };
}

export function matchTimes(
  base: CameraTime | null | undefined,
  match: PhotoMatch,
  zone: string
): string | null {
  if (!base || match.time === null) return null;
  const after = clockOfDay(match.time, zone);
  return base.instant === match.time ? after : `${clockOfDay(base.instant, zone)} → ${after}`;
}

export const MATCH_TONES = {
  ok: 'bg-primary/10 text-primary',
  out: 'bg-warning-500/10 text-warning-800',
  kept: 'bg-success-500/10 text-success-800'
} as const satisfies Record<MatchTone, string>;
