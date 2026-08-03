import { describe, expect, it } from 'vitest';
import { neutralEdits } from '$lib/types/edits';
import {
  displayUvToSceneUv,
  sceneUvToDisplayUv,
  steppedSegment,
  viewTransform
} from './canvasCoords';

describe('steppedSegment', () => {
  it.each([
    [0, 0, 1, 0, 0.25, 4],
    [0, 0, 0, 0, 0.25, 1],
    [0, 0, 0.1, 0, 1, 1]
  ])('splits (%s,%s)->(%s,%s) at step %s into %s points', (ax, ay, bx, by, step, n) => {
    const pts = steppedSegment(ax, ay, bx, by, step);
    expect(pts).toHaveLength(n);
    expect(pts[pts.length - 1][0]).toBeCloseTo(bx, 6);
    expect(pts[pts.length - 1][1]).toBeCloseTo(by, 6);
  });
});

describe('view transform round trip', () => {
  it.each([
    [0 as const, false, false],
    [90 as const, false, false],
    [180 as const, true, false]
  ])('maps display uv back to itself for rotate %s', (rotate, flipH, flipV) => {
    const edits = neutralEdits();
    edits.geometry.rotate = rotate;
    edits.geometry.flip_h = flipH;
    edits.geometry.flip_v = flipV;
    edits.geometry.crop = { x: 0.1, y: 0.2, w: 0.6, h: 0.5 };
    const view = viewTransform(edits, {
      asset_id: 'a',
      width: 600,
      height: 400,
      source_w: 1000,
      source_h: 800,
      renderer: 'cpu',
      histogram: { r: [], g: [], b: [], l: [] }
    });
    const scene = displayUvToSceneUv(view, 0.3, 0.7);
    const back = sceneUvToDisplayUv(view, scene[0], scene[1]);
    expect(back[0]).toBeCloseTo(0.3, 5);
    expect(back[1]).toBeCloseTo(0.7, 5);
  });
});
