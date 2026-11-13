import { describe, expect, it } from 'vitest';
import { cropGuides, GUIDE_TURNS, mirrorTurn, turnPoint } from './cropGuides';

function ends(paths: ReturnType<typeof cropGuides>): string[] {
  return paths.map((path) => {
    const first = path[0] ?? [0, 0];
    const last = path.at(-1) ?? [0, 0];
    return `${first[0].toFixed(3)},${first[1].toFixed(3)}>${last[0].toFixed(3)},${last[1].toFixed(3)}`;
  });
}

describe('crop guides', () => {
  it('draws two lines per third on each axis', () => {
    expect(cropGuides('thirds', 0, 1.5)).toHaveLength(4);
    expect(cropGuides('grid', 0, 1.5)).toHaveLength(14);
    expect(cropGuides('off', 0, 1.5)).toEqual([]);
  });

  it('ignores the turn for symmetric guides', () => {
    expect(cropGuides('golden', 3, 1.5)).toEqual(cropGuides('golden', 0, 1.5));
    expect(cropGuides('diagonal', 5, 1.5)).toEqual(cropGuides('diagonal', 0, 1.5));
  });

  it('drops perpendiculars onto the diagonal in pixel space', () => {
    const [, upper] = cropGuides('triangle', 0, 2);
    const foot = upper?.[1] ?? [0, 0];
    expect(foot[0]).toBeCloseTo(0.8);
    expect(foot[1]).toBeCloseTo(0.8);
    const dx = (foot[0] - 1) * 2;
    const dy = foot[1];
    expect(dx * 2 + dy * 1).toBeCloseTo(0);
  });

  it('keeps the triangle perpendicular when the turn transposes it', () => {
    const [diagonal, upper] = cropGuides('triangle', 4, 2);
    const from = upper?.[0] ?? [0, 0];
    const foot = upper?.[1] ?? [0, 0];
    const a = diagonal?.[0] ?? [0, 0];
    const b = diagonal?.[1] ?? [0, 0];
    const along = [(b[0] - a[0]) * 2, b[1] - a[1]];
    const across = [(foot[0] - from[0]) * 2, foot[1] - from[1]];
    expect((along[0] ?? 0) * (across[0] ?? 0) + (along[1] ?? 0) * (across[1] ?? 0)).toBeCloseTo(0);
  });

  it('draws one continuous spiral from a corner', () => {
    const [path] = cropGuides('spiral', 0, 1.618);
    expect(path?.[0]?.[0]).toBeCloseTo(0);
    expect(path?.[0]?.[1]).toBeCloseTo(1);
    const steps = (path ?? []).slice(1).map((point, i) => {
      const prev = path?.[i] ?? point;
      return Math.hypot(point[0] - prev[0], point[1] - prev[1]);
    });
    expect(Math.max(...steps)).toBeLessThan(0.1);
  });

  it('turns the spiral through eight distinct orientations', () => {
    const seen = new Set(
      Array.from({ length: GUIDE_TURNS }, (_, turn) => ends(cropGuides('spiral', turn, 1.5))[0])
    );
    expect(seen.size).toBe(GUIDE_TURNS);
  });

  it('mirrors every turn left to right with its odd partner', () => {
    for (let turn = 0; turn < GUIDE_TURNS; turn++) {
      const mirrored = mirrorTurn(turn);
      expect(mirrorTurn(mirrored)).toBe(turn);
      for (const point of [
        [0.2, 0.3],
        [0.9, 0.6]
      ] as const) {
        const [x, y] = turnPoint(point, turn);
        const [mx, my] = turnPoint(point, mirrored);
        expect(mx).toBeCloseTo(1 - x);
        expect(my).toBeCloseTo(y);
      }
    }
  });

  it('maps the unit square onto itself', () => {
    for (let turn = 0; turn < GUIDE_TURNS; turn++) {
      const corners = [
        [0, 0],
        [1, 0],
        [0, 1],
        [1, 1]
      ].map(([x, y]) => turnPoint([x ?? 0, y ?? 0], turn).join(','));
      expect(new Set(corners)).toEqual(new Set(['0,0', '1,0', '0,1', '1,1']));
    }
  });
});
