import type { TrackPoint } from '#lib/geotag/trackFile.ts';
import { distanceMetres, SAME_SPOT_METRES } from '#lib/geotag/spots.ts';

export type OutsideReason = 'before' | 'after' | 'gap';

const STEADY_RATIO = 2;
const MOVING_MPS = 0.5;

export type Located =
  | { kind: 'placed'; lat: number; lon: number; gapMs: number }
  | { kind: 'outside'; reason: OutsideReason; gapMs: number };

export interface LocateOptions {
  toleranceMs: number;
  interpolate: boolean;
}

export class TrackLog {
  readonly times: Float64Array;
  readonly lats: Float64Array;
  readonly lons: Float64Array;
  readonly segments: Int32Array;

  constructor(segments: readonly (readonly TrackPoint[])[]) {
    const flat = segments
      .flatMap((points, segment) => points.map((point) => ({ ...point, segment })))
      .sort((a, b) => a.time - b.time);
    this.times = Float64Array.from(flat, (point) => point.time);
    this.lats = Float64Array.from(flat, (point) => point.lat);
    this.lons = Float64Array.from(flat, (point) => point.lon);
    this.segments = Int32Array.from(flat, (point) => point.segment);
  }

  get size(): number {
    return this.times.length;
  }

  get start(): number {
    return this.times[0] ?? Number.NaN;
  }

  get end(): number {
    return this.times[this.times.length - 1] ?? Number.NaN;
  }

  after(time: number): number {
    let low = 0;
    let high = this.times.length;
    while (low < high) {
      const mid = (low + high) >>> 1;
      if ((this.times[mid] ?? 0) < time) low = mid + 1;
      else high = mid;
    }
    return low;
  }

  nearestGap(time: number): number {
    const index = this.after(time);
    const next = this.times[index];
    const prev = this.times[index - 1];
    return Math.min(
      next === undefined ? Infinity : next - time,
      prev === undefined ? Infinity : time - prev
    );
  }

  #at(index: number): { lat: number; lon: number } {
    return { lat: this.lats[index] ?? 0, lon: this.lons[index] ?? 0 };
  }

  #speed(from: number, to: number): number | null {
    const start = this.times[from];
    const end = this.times[to];
    if (start === undefined || end === undefined || end <= start) return null;
    if (this.segments[from] !== this.segments[to]) return null;
    return distanceMetres(this.#at(from), this.#at(to)) / ((end - start) / 1000);
  }

  #steady(before: number, after: number): boolean {
    const across = this.#speed(before, after);
    const around = [this.#speed(before - 1, before), this.#speed(after, after + 1)].filter(
      (speed): speed is number => speed !== null && speed >= MOVING_MPS
    );
    if (across === null || around.length === 0) return false;
    const pace = around.reduce((sum, speed) => sum + speed, 0) / around.length;
    return across >= pace / STEADY_RATIO && across <= pace * STEADY_RATIO;
  }

  #point(index: number, gapMs: number): Located {
    return { kind: 'placed', lat: this.lats[index] ?? 0, lon: this.lons[index] ?? 0, gapMs };
  }

  locate(time: number, options: LocateOptions): Located {
    const count = this.times.length;
    if (count === 0) return { kind: 'outside', reason: 'gap', gapMs: Infinity };
    const index = this.after(time);
    const next = this.times[index];
    if (next === time) return this.#point(index, 0);
    const prev = this.times[index - 1];
    if (prev === undefined && next !== undefined) {
      const gap = next - time;
      return gap <= options.toleranceMs
        ? this.#point(index, gap)
        : { kind: 'outside', reason: 'before', gapMs: gap };
    }
    if (next === undefined && prev !== undefined) {
      const gap = time - prev;
      return gap <= options.toleranceMs
        ? this.#point(index - 1, gap)
        : { kind: 'outside', reason: 'after', gapMs: gap };
    }
    if (prev === undefined || next === undefined) {
      return { kind: 'outside', reason: 'gap', gapMs: Infinity };
    }
    const before = index - 1;
    const sameSegment = this.segments[before] === this.segments[index];
    const span = next - prev;
    const a = { lat: this.lats[before] ?? 0, lon: this.lons[before] ?? 0 };
    const b = { lat: this.lats[index] ?? 0, lon: this.lons[index] ?? 0 };
    const gap = Math.min(time - prev, next - time);
    const bridgeable =
      span <= 2 * options.toleranceMs ||
      distanceMetres(a, b) <= SAME_SPOT_METRES ||
      this.#steady(before, index);
    if (options.interpolate && sameSegment && bridgeable) {
      const share = (time - prev) / span;
      return {
        kind: 'placed',
        lat: a.lat + (b.lat - a.lat) * share,
        lon: a.lon + (b.lon - a.lon) * share,
        gapMs: gap
      };
    }
    if (gap <= options.toleranceMs)
      return this.#point(time - prev <= next - time ? before : index, gap);
    return { kind: 'outside', reason: 'gap', gapMs: gap };
  }

  path(): [number, number][][] {
    const lines: [number, number][][] = [];
    let current: [number, number][] = [];
    let segment = -1;
    for (let i = 0; i < this.times.length; i += 1) {
      const id = this.segments[i] ?? 0;
      if (id !== segment && current.length > 0) {
        lines.push(current);
        current = [];
      }
      segment = id;
      current.push([this.lons[i] ?? 0, this.lats[i] ?? 0]);
    }
    if (current.length > 0) lines.push(current);
    return lines;
  }

  lengthMetres(): number {
    let total = 0;
    for (let i = 1; i < this.times.length; i += 1) {
      if (this.segments[i] !== this.segments[i - 1]) continue;
      total += distanceMetres(
        { lat: this.lats[i - 1] ?? 0, lon: this.lons[i - 1] ?? 0 },
        { lat: this.lats[i] ?? 0, lon: this.lons[i] ?? 0 }
      );
    }
    return total;
  }
}
