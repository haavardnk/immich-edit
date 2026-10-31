import type { MaskRange } from '#lib/api/masks.ts';

const LEVELS = 256;

function smoothstep(edge0: number, edge1: number, x: number): number {
  if (edge1 - edge0 <= 1e-6) return x < edge0 ? 0 : 1;
  const t = Math.min(1, Math.max(0, (x - edge0) / (edge1 - edge0)));
  return t * t * (3 - 2 * t);
}

function rangeWeight(value: number, lo: number, hi: number, softness: number): number {
  const soft = Math.max(0, softness);
  if (soft <= 1e-6) return value >= lo && value <= hi ? 1 : 0;
  return Math.min(smoothstep(lo - soft, lo, value), 1 - smoothstep(hi, hi + soft, value));
}

export function windowLevels(levels: Uint8Array, range: MaskRange): Uint8Array {
  const counts = new Uint32Array(LEVELS);
  for (const v of levels) counts[v] = (counts[v] ?? 0) + 1;
  const total = Math.max(1, levels.length);
  const lo = Math.min(range.min, range.max);
  const hi = Math.max(range.min, range.max);
  const lut = new Uint8Array(LEVELS);
  let covered = 0;
  for (let level = LEVELS - 1; level >= 0; level--) {
    const start = covered / total;
    covered += counts[level] ?? 0;
    const end = covered / total;
    const rank = start <= lo && end >= hi ? lo : end;
    lut[level] = Math.floor(rangeWeight(rank, lo, hi, range.softness) * 255 + 0.5);
  }
  return levels.map((v) => lut[v] ?? 0);
}
