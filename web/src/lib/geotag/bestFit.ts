import type { TrackLog } from '#lib/geotag/trackLog.ts';

const SECOND = 1000;
const MINUTE = 60 * SECOND;
const HOUR = 60 * MINUTE;
const RANGE = 14 * HOUR;
const ZONE_STEP = 30 * MINUTE;

export interface Fit {
  correctionMs: number;
  matched: number;
}

function matched(bases: readonly number[], log: TrackLog, shift: number, tolerance: number) {
  let count = 0;
  for (const base of bases) if (log.nearestGap(base + shift) <= tolerance) count += 1;
  return count;
}

interface Run {
  from: number;
  to: number;
}

function bestRuns(scores: readonly number[], top: number): Run[] {
  const runs: Run[] = [];
  let open: number | null = null;
  scores.forEach((score, index) => {
    if (score === top && open === null) open = index;
    if (score !== top && open !== null) {
      runs.push({ from: open, to: index - 1 });
      open = null;
    }
  });
  if (open !== null) runs.push({ from: open, to: scores.length - 1 });
  return runs;
}

function shiftAt(index: number): number {
  return -RANGE + index * MINUTE;
}

function middle(run: Run): number {
  return Math.round((shiftAt(run.from) + shiftAt(run.to)) / 2 / MINUTE) * MINUTE;
}

function zoneStep(run: Run): number | null {
  const centre = middle(run);
  const below = Math.floor(centre / ZONE_STEP) * ZONE_STEP;
  const steps = [below, below + ZONE_STEP]
    .filter((shift) => shift >= shiftAt(run.from) && shift <= shiftAt(run.to))
    .sort((a, b) => Math.abs(a - centre) - Math.abs(b - centre) || Math.abs(a) - Math.abs(b));
  return steps[0] ?? null;
}

function wider(a: Run, b: Run): Run {
  const widthA = a.to - a.from;
  const widthB = b.to - b.from;
  if (widthA !== widthB) return widthA > widthB ? a : b;
  return Math.abs(middle(a)) <= Math.abs(middle(b)) ? a : b;
}

export function bestFit(bases: readonly number[], log: TrackLog, toleranceMs: number): Fit | null {
  if (bases.length === 0 || log.size === 0) return null;
  const scores: number[] = [];
  for (let shift = -RANGE; shift <= RANGE; shift += MINUTE) {
    scores.push(matched(bases, log, shift, toleranceMs));
  }
  const top = Math.max(...scores);
  if (top === 0) return null;
  const runs = bestRuns(scores, top);
  if (runs.some((run) => shiftAt(run.from) <= 0 && shiftAt(run.to) >= 0)) {
    return { correctionMs: 0, matched: top };
  }
  const stepped = runs.filter((run) => zoneStep(run) !== null);
  if (stepped.length > 0) {
    return { correctionMs: zoneStep(stepped.reduce(wider)) ?? 0, matched: top };
  }
  return { correctionMs: middle(runs.reduce(wider)), matched: top };
}
