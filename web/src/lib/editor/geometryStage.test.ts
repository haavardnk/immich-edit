import { neutralPerspective } from '#lib/utils/perspective.ts';
import { describe, expect, it } from 'vitest';
import {
  outputToStage,
  sourceToStage,
  stageFrame,
  stageToOutput,
  stageToSource,
  type StageFrameInput
} from './geometryStage';

const base: StageFrameInput = {
  bboxW: 600,
  bboxH: 400,
  orientedW: 600,
  orientedH: 400,
  angle: 0,
  perspective: neutralPerspective()
};

describe('geometryStage', () => {
  it('maps uv corners onto the stage box without rotation', () => {
    const frame = stageFrame(base);
    expect(outputToStage(frame, [0, 0])).toEqual([0, 0]);
    expect(outputToStage(frame, [1, 1])).toEqual([600, 400]);
  });

  it('rotates about the stage centre', () => {
    const frame = stageFrame({ ...base, bboxW: 400, bboxH: 400, orientedW: 400, angle: 90 });
    const [x, y] = outputToStage(frame, [1, 0.5]);
    expect(x).toBeCloseTo(200);
    expect(y).toBeCloseTo(400);
  });

  it.each([
    ['plain', base],
    ['rotated', { ...base, bboxW: 700, bboxH: 520, angle: 12 }],
    [
      'keystoned',
      {
        ...base,
        bboxW: 650,
        bboxH: 480,
        angle: -7,
        perspective: { ...neutralPerspective(), vertical: 30, horizontal: -15, aspect: 10 }
      }
    ]
  ])('round-trips stage points through output and source uv (%s)', (_, input) => {
    const frame = stageFrame(input);
    const point: [number, number] = [210, 130];
    const out = outputToStage(frame, stageToOutput(frame, point));
    const src = sourceToStage(frame, stageToSource(frame, point));
    expect(out[0]).toBeCloseTo(point[0], 6);
    expect(out[1]).toBeCloseTo(point[1], 6);
    expect(src[0]).toBeCloseTo(point[0], 6);
    expect(src[1]).toBeCloseTo(point[1], 6);
  });
});
