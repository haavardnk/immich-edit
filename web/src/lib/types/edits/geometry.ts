import { perspectiveIsIdentity, type PerspectiveEdits } from '$lib/utils/perspective';

export interface CropRect {
  x: number;
  y: number;
  w: number;
  h: number;
}

export type AspectLock =
  { kind: 'original' } | { kind: 'free' } | { kind: 'ratio'; num: number; den: number };

export interface GeometryEdits {
  rotate: 0 | 90 | 180 | 270;
  rotate_angle: number;
  flip_h: boolean;
  flip_v: boolean;
  crop: CropRect | null;
  aspect: AspectLock;
  perspective: PerspectiveEdits | null;
}

export const FULL_CROP: CropRect = { x: 0, y: 0, w: 1, h: 1 };

export function isFullCrop(c: CropRect | null): boolean {
  if (!c) return true;
  return (
    Math.abs(c.x) < 1e-4 &&
    Math.abs(c.y) < 1e-4 &&
    Math.abs(c.w - 1) < 1e-4 &&
    Math.abs(c.h - 1) < 1e-4
  );
}

export function neutralGeometry(): GeometryEdits {
  return {
    rotate: 0,
    rotate_angle: 0,
    flip_h: false,
    flip_v: false,
    crop: null,
    aspect: { kind: 'original' },
    perspective: null
  };
}

export function geometryIsIdentity(g: GeometryEdits): boolean {
  return (
    g.rotate === 0 &&
    Math.abs(g.rotate_angle) < 1e-4 &&
    !g.flip_h &&
    !g.flip_v &&
    isFullCrop(g.crop) &&
    g.aspect.kind === 'original' &&
    perspectiveIsIdentity(g.perspective)
  );
}
