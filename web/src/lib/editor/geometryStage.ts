import { degToRad } from '#lib/utils/geom.ts';
import {
  mat3Apply,
  perspectiveForward,
  perspectiveInverse,
  type Mat3,
  type PerspectiveEdits,
  type Point
} from '#lib/utils/perspective.ts';

interface StageBox {
  bboxW: number;
  bboxH: number;
  orientedW: number;
  orientedH: number;
}

export interface StageFrame extends StageBox {
  cos: number;
  sin: number;
  forward: Mat3;
  inverse: Mat3;
}

export interface StageFrameInput extends StageBox {
  angle: number;
  perspective: PerspectiveEdits;
}

export function stageFrame({ angle, perspective, ...box }: StageFrameInput): StageFrame {
  const a = degToRad(angle);
  return {
    ...box,
    cos: Math.cos(a),
    sin: Math.sin(a),
    forward: perspectiveForward(perspective),
    inverse: perspectiveInverse(perspective)
  };
}

export function outputToStage(frame: StageFrame, uv: Point): Point {
  const px = (uv[0] - 0.5) * frame.orientedW;
  const py = (uv[1] - 0.5) * frame.orientedH;
  return [
    px * frame.cos - py * frame.sin + frame.bboxW / 2,
    px * frame.sin + py * frame.cos + frame.bboxH / 2
  ];
}

export function stageToOutput(frame: StageFrame, point: Point): Point {
  const dx = point[0] - frame.bboxW / 2;
  const dy = point[1] - frame.bboxH / 2;
  const px = dx * frame.cos + dy * frame.sin;
  const py = -dx * frame.sin + dy * frame.cos;
  return [px / Math.max(frame.orientedW, 1) + 0.5, py / Math.max(frame.orientedH, 1) + 0.5];
}

export function sourceToStage(frame: StageFrame, uv: Point): Point {
  return outputToStage(frame, mat3Apply(frame.forward, uv));
}

export function stageToSource(frame: StageFrame, point: Point): Point {
  return mat3Apply(frame.inverse, stageToOutput(frame, point));
}
