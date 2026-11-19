import type { CropRect } from '#lib/types/edits.ts';
import type { PerspectiveEdits } from '#lib/utils/perspective.ts';
import {
  ready,
  updateDraftAngle,
  updateDraftPerspective,
  type GeometryCtx,
  type GeometrySession
} from './geometry.svelte';

export type DialId = 'angle' | 'vertical' | 'horizontal' | 'aspect';

type Held = {
  sess: GeometrySession;
  angle: number;
  perspective: PerspectiveEdits;
  crop: CropRect;
  userEditedCrop: boolean;
};

let held: Held | null = null;

export function dialValue(sess: GeometrySession, id: DialId): number {
  return id === 'angle' ? sess.draftAngle : sess.draftPerspective[id];
}

export function setDialValue(ctx: GeometryCtx, id: DialId, value: number): void {
  if (id === 'angle') updateDraftAngle(ctx, value);
  else updateDraftPerspective(ctx, { [id]: value });
}

export function holdDial(ctx: GeometryCtx, id: DialId): void {
  const sess = ready(ctx);
  if (!sess || held || Math.abs(dialValue(sess, id)) < 1e-4) return;
  held = {
    sess,
    angle: sess.draftAngle,
    perspective: { ...sess.draftPerspective },
    crop: { ...sess.draftCrop },
    userEditedCrop: sess.userEditedCrop
  };
  setDialValue(ctx, id, 0);
}

export function releaseDial(ctx: GeometryCtx): void {
  const snapshot = held;
  held = null;
  if (!snapshot || ctx.geometrySession !== snapshot.sess) return;
  const sess = snapshot.sess;
  sess.draftAngle = snapshot.angle;
  sess.draftPerspective = snapshot.perspective;
  sess.draftCrop = snapshot.crop;
  sess.userEditedCrop = snapshot.userEditedCrop;
}
