import type { MaskComponentKind, Vec2f } from '$lib/types/edits';
import { clamp01, degToRad } from '$lib/utils/geom';

type RadialKind = Extract<MaskComponentKind, { kind: 'radial' }>;

export interface DragContext {
  aspect: number;
}

export type DragKind =
  | { kind: 'linear-p0' }
  | { kind: 'linear-p1' }
  | { kind: 'linear-move'; startP0: Vec2f; startP1: Vec2f; downAtN: Vec2f }
  | { kind: 'linear-feather' }
  | { kind: 'radial-center' }
  | { kind: 'radial-rx'; sign: 1 | -1 }
  | { kind: 'radial-ry'; sign: 1 | -1 }
  | { kind: 'radial-feather' }
  | { kind: 'radial-rotate' }
  | { kind: 'polygon-vertex'; index: number }
  | { kind: 'polygon-move'; start: Vec2f[]; downAtN: Vec2f };

export function normalizeDegrees(deg: number): number {
  return ((((deg + 180) % 360) + 360) % 360) - 180;
}

export function radialAxes(kind: RadialKind, aspect: number): { x: Vec2f; y: Vec2f } {
  const t = degToRad(kind.angle ?? 0);
  const c = Math.cos(t);
  const s = Math.sin(t);
  const { x: rx, y: ry } = kind.radius_xy;
  return { x: { x: rx * c, y: rx * aspect * s }, y: { x: (-ry * s) / aspect, y: ry * c } };
}

function radialLocal(
  kind: RadialKind,
  n: Vec2f,
  aspect: number
): { along: number; across: number } {
  const t = degToRad(kind.angle ?? 0);
  const c = Math.cos(t);
  const s = Math.sin(t);
  const dx = (n.x - kind.center.x) * aspect;
  const dy = n.y - kind.center.y;
  return { along: dx * c + dy * s, across: -dx * s + dy * c };
}

function withAngle(kind: RadialKind, deg: number): RadialKind {
  const angle = normalizeDegrees(deg);
  const plain: RadialKind = {
    kind: 'radial',
    center: kind.center,
    radius_xy: kind.radius_xy,
    feather: kind.feather
  };
  return angle === 0 ? plain : { ...plain, angle };
}

function draggedRadial(
  kind: RadialKind,
  drag: DragKind,
  n: Vec2f,
  aspect: number
): RadialKind | null {
  if (drag.kind === 'radial-center') return { ...kind, center: n };
  const { along, across } = radialLocal(kind, n, aspect);
  if (drag.kind === 'radial-rx') {
    const rx = Math.max(0.005, Math.abs(along) / aspect);
    return { ...kind, radius_xy: { x: rx, y: kind.radius_xy.y } };
  }
  if (drag.kind === 'radial-ry') {
    const ry = Math.max(0.005, Math.abs(across));
    return { ...kind, radius_xy: { x: kind.radius_xy.x, y: ry } };
  }
  if (drag.kind === 'radial-feather') {
    const ex = kind.radius_xy.x < 1e-6 ? 0 : along / (kind.radius_xy.x * aspect);
    const ey = kind.radius_xy.y < 1e-6 ? 0 : across / kind.radius_xy.y;
    return { ...kind, feather: clamp01(1 - Math.hypot(ex, ey)) };
  }
  if (drag.kind === 'radial-rotate') {
    const dx = (n.x - kind.center.x) * aspect;
    const dy = n.y - kind.center.y;
    if (Math.hypot(dx, dy) < 1e-6) return null;
    return withAngle(kind, (Math.atan2(dy, dx) * 180) / Math.PI);
  }
  return null;
}

export function draggedKind(
  kind: MaskComponentKind,
  drag: DragKind,
  n: Vec2f,
  { aspect }: DragContext = { aspect: 1 }
): MaskComponentKind | null {
  if (kind.kind === 'linear') {
    if (drag.kind === 'linear-p0') return { ...kind, p0: n };
    if (drag.kind === 'linear-p1') return { ...kind, p1: n };
    if (drag.kind === 'linear-move') {
      const dx = n.x - drag.downAtN.x;
      const dy = n.y - drag.downAtN.y;
      return {
        ...kind,
        p0: { x: clamp01(drag.startP0.x + dx), y: clamp01(drag.startP0.y + dy) },
        p1: { x: clamp01(drag.startP1.x + dx), y: clamp01(drag.startP1.y + dy) }
      };
    }
    if (drag.kind === 'linear-feather') {
      const dx = kind.p1.x - kind.p0.x;
      const dy = kind.p1.y - kind.p0.y;
      const len2 = Math.max(1e-9, dx * dx + dy * dy);
      const mx = (kind.p0.x + kind.p1.x) * 0.5;
      const my = (kind.p0.y + kind.p1.y) * 0.5;
      const t = ((n.x - mx) * dx + (n.y - my) * dy) / len2;
      return { ...kind, feather: clamp01(2 * Math.abs(t)) };
    }
    return null;
  }

  if (kind.kind === 'radial') return draggedRadial(kind, drag, n, aspect);

  if (kind.kind === 'polygon') {
    if (drag.kind === 'polygon-vertex') {
      const index = drag.index;
      return { ...kind, points: kind.points.map((p, i) => (i === index ? n : p)) };
    }
    if (drag.kind === 'polygon-move') {
      const dx = n.x - drag.downAtN.x;
      const dy = n.y - drag.downAtN.y;
      return {
        ...kind,
        points: drag.start.map((p) => ({ x: clamp01(p.x + dx), y: clamp01(p.y + dy) }))
      };
    }
    return null;
  }

  return null;
}

function moveDrag(kind: MaskComponentKind): { anchor: Vec2f; drag: DragKind } | null {
  if (kind.kind === 'linear') {
    const anchor = { x: (kind.p0.x + kind.p1.x) / 2, y: (kind.p0.y + kind.p1.y) / 2 };
    return {
      anchor,
      drag: { kind: 'linear-move', startP0: kind.p0, startP1: kind.p1, downAtN: anchor }
    };
  }
  if (kind.kind === 'radial') return { anchor: kind.center, drag: { kind: 'radial-center' } };
  if (kind.kind === 'polygon') {
    const anchor = kind.points[0];
    if (!anchor) return null;
    return { anchor, drag: { kind: 'polygon-move', start: kind.points, downAtN: anchor } };
  }
  return null;
}

export function nudgeable(kind: MaskComponentKind): boolean {
  return moveDrag(kind) !== null;
}

export function nudgedKind(
  kind: MaskComponentKind,
  dx: number,
  dy: number,
  toPx: (v: Vec2f) => { x: number; y: number },
  fromPx: (x: number, y: number) => Vec2f
): MaskComponentKind | null {
  const move = moveDrag(kind);
  if (!move) return null;
  const at = toPx(move.anchor);
  return draggedKind(kind, move.drag, fromPx(at.x + dx, at.y + dy));
}
