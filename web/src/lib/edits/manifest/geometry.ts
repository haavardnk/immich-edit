import {
  isFullCrop,
  type AspectLock,
  type CropRect,
  type Edits,
  type GeometryEdits
} from '$lib/types/edits';
import {
  clampPerspective,
  neutralPerspective,
  perspectiveIsIdentity,
  type PerspectiveEdits
} from '$lib/utils/perspective';

export function encodeTransform(edits: Edits, ops: Record<string, unknown>): void {
  const geometry = edits.geometry;
  const cropActive = !isFullCrop(geometry.crop);
  const angleActive = Math.abs(geometry.rotate_angle) > 1e-4;
  const aspectActive = geometry.aspect.kind !== 'original';
  const rotateActive = geometry.rotate !== 0;
  const flipActive = geometry.flip_h || geometry.flip_v;
  const perspectiveActive = !perspectiveIsIdentity(geometry.perspective);
  if (
    !cropActive &&
    !angleActive &&
    !aspectActive &&
    !rotateActive &&
    !flipActive &&
    !perspectiveActive
  )
    return;
  const obj: Record<string, unknown> = {};
  if (rotateActive) obj.rotate = geometry.rotate;
  if (geometry.flip_h) obj.flip_h = true;
  if (geometry.flip_v) obj.flip_v = true;
  if (angleActive) obj.angle = geometry.rotate_angle;
  if (geometry.crop && cropActive) obj.crop = geometry.crop;
  if (perspectiveActive && geometry.perspective)
    obj.perspective = clampPerspective(geometry.perspective);
  obj.aspect = geometry.aspect;
  ops.transform = obj;
}

export function decodeTransform(ops: Record<string, unknown>, edits: Edits): void {
  const transform = ops.transform as
    | {
        rotate?: number;
        flip_h?: boolean;
        flip_v?: boolean;
        angle?: number;
        crop?: CropRect;
        aspect?: AspectLock;
        perspective?: PerspectiveEdits;
      }
    | undefined;
  if (!transform) return;
  if (transform.rotate !== undefined)
    edits.geometry.rotate = transform.rotate as GeometryEdits['rotate'];
  if (transform.flip_h !== undefined) edits.geometry.flip_h = transform.flip_h;
  if (transform.flip_v !== undefined) edits.geometry.flip_v = transform.flip_v;
  if (transform.angle !== undefined) edits.geometry.rotate_angle = transform.angle;
  if (transform.crop) edits.geometry.crop = transform.crop;
  if (transform.aspect) edits.geometry.aspect = transform.aspect;
  if (transform.perspective) {
    const perspective = clampPerspective({ ...neutralPerspective(), ...transform.perspective });
    edits.geometry.perspective = perspectiveIsIdentity(perspective) ? null : perspective;
  }
}
