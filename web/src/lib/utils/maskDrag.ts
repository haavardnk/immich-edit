import type { MaskComponentKind, Vec2f } from '$lib/types/edits';
import { clamp01 } from '$lib/utils/geom';

export type DragKind =
  | { kind: 'linear-p0' }
  | { kind: 'linear-p1' }
  | { kind: 'linear-move'; startP0: Vec2f; startP1: Vec2f; downAtN: Vec2f }
  | { kind: 'linear-feather' }
  | { kind: 'radial-center' }
  | { kind: 'radial-rx'; sign: 1 | -1 }
  | { kind: 'radial-ry'; sign: 1 | -1 }
  | { kind: 'radial-feather' }
  | { kind: 'polygon-vertex'; index: number }
  | { kind: 'polygon-move'; start: Vec2f[]; downAtN: Vec2f };

export function draggedKind(
  kind: MaskComponentKind,
  drag: DragKind,
  n: Vec2f
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

  if (kind.kind === 'radial') {
    if (drag.kind === 'radial-center') return { ...kind, center: n };
    if (drag.kind === 'radial-rx') {
      const rx = Math.max(0.005, Math.abs(n.x - kind.center.x));
      return { ...kind, radius_xy: { x: rx, y: kind.radius_xy.y } };
    }
    if (drag.kind === 'radial-ry') {
      const ry = Math.max(0.005, Math.abs(n.y - kind.center.y));
      return { ...kind, radius_xy: { x: kind.radius_xy.x, y: ry } };
    }
    if (drag.kind === 'radial-feather') {
      const ex = kind.radius_xy.x < 1e-6 ? 0 : (n.x - kind.center.x) / kind.radius_xy.x;
      const ey = kind.radius_xy.y < 1e-6 ? 0 : (n.y - kind.center.y) / kind.radius_xy.y;
      return { ...kind, feather: clamp01(1 - Math.hypot(ex, ey)) };
    }
    return null;
  }

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
