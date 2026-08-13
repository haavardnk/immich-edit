import { FULL_CROP, type AspectLock, type CropRect, type Edits } from '$lib/types/edits';
import { livePreview } from '$lib/api/preview';
import { makeObjectUrl, revoke } from '$lib/utils/object-url';
import {
  aspectRatioFor,
  constrainCropRect,
  largestInscribedRect,
  refitCropAtAspect
} from '$lib/utils/geom';
import {
  clampPerspective,
  limitPerspective,
  neutralPerspective,
  perspectiveInverse,
  perspectiveIsIdentity,
  type Mat3,
  type PerspectiveEdits
} from '$lib/utils/perspective';

export interface GeometrySession {
  id: number;
  pinnedUrl: string | null;
  pinnedReady: boolean;
  srcW: number;
  srcH: number;
  draftRotate: 0 | 90 | 180 | 270;
  draftFlipH: boolean;
  draftFlipV: boolean;
  draftAngle: number;
  draftCrop: CropRect;
  draftAspect: AspectLock;
  draftPerspective: PerspectiveEdits;
  userEditedCrop: boolean;
}

export interface GeometryCtx {
  assetId: string | null;
  initialised: boolean;
  edits: Edits;
  geometrySession: GeometrySession | null;
  error: string | null;
  clearView(): void;
  onCommit(action?: string): Promise<void>;
}

const LIVE_EDGE = 1600;
let nextSessionId = 0;

function perspInv(sess: GeometrySession): Mat3 {
  return perspectiveInverse(sess.draftPerspective);
}

function sourceDims(sess: GeometrySession): { sw: number; sh: number } {
  const swapped = sess.draftRotate === 90 || sess.draftRotate === 270;
  return { sw: swapped ? sess.srcH : sess.srcW, sh: swapped ? sess.srcW : sess.srcH };
}

function refitDraftCrop(sess: GeometrySession): void {
  const { sw, sh } = sourceDims(sess);
  const angle = sess.draftAngle;
  const persp = perspInv(sess);
  const ratio = aspectRatioFor(sess.draftAspect, sw, sh);
  if (ratio !== null) {
    sess.draftCrop = refitCropAtAspect(sess.draftCrop, sw, sh, angle, ratio, persp);
    return;
  }
  if (!sess.userEditedCrop) {
    sess.draftCrop = largestInscribedRect(sw, sh, angle, sw / sh, persp);
    return;
  }
  sess.draftCrop = constrainCropRect(sess.draftCrop, sess.draftCrop, sw, sh, angle, persp);
}

async function loadPinnedPreview(
  ctx: GeometryCtx,
  baseEdits: Edits,
  sessionId: number
): Promise<void> {
  if (!ctx.assetId) return;
  const canonical: Edits = {
    ...baseEdits,
    geometry: {
      ...baseEdits.geometry,
      rotate: 0,
      flip_h: false,
      flip_v: false,
      rotate_angle: 0,
      crop: null,
      perspective: null
    }
  };
  let url: string | null = null;
  try {
    const { blob } = await livePreview(ctx.assetId, canonical, LIVE_EDGE, 'none');
    url = makeObjectUrl(blob);
    const dims = await new Promise<{ w: number; h: number }>((resolve, reject) => {
      const img = new Image();
      img.onload = () => resolve({ w: img.naturalWidth, h: img.naturalHeight });
      img.onerror = () => reject(new Error('pinned preview decode failed'));
      img.src = url as string;
    });
    const sess = ctx.geometrySession;
    if (sess?.id !== sessionId || dims.w <= 0 || dims.h <= 0) {
      revoke(url);
      return;
    }
    if (sess.pinnedUrl) revoke(sess.pinnedUrl);
    sess.pinnedUrl = url;
    sess.srcW = dims.w;
    sess.srcH = dims.h;
    sess.pinnedReady = true;
  } catch (e) {
    if (url) revoke(url);
    if (ctx.geometrySession?.id !== sessionId) return;
    ctx.error = e instanceof Error ? e.message : String(e);
  }
}

export function startSession(ctx: GeometryCtx): void {
  if (!ctx.assetId || !ctx.initialised || ctx.geometrySession) return;
  ctx.clearView();
  const baseEdits = $state.snapshot(ctx.edits) as Edits;
  const sessionId = ++nextSessionId;
  ctx.geometrySession = {
    id: sessionId,
    pinnedUrl: null,
    pinnedReady: false,
    srcW: 0,
    srcH: 0,
    draftRotate: baseEdits.geometry.rotate,
    draftFlipH: baseEdits.geometry.flip_h,
    draftFlipV: baseEdits.geometry.flip_v,
    draftAngle: baseEdits.geometry.rotate_angle,
    draftCrop: baseEdits.geometry.crop ?? FULL_CROP,
    draftAspect: baseEdits.geometry.aspect,
    draftPerspective: baseEdits.geometry.perspective ?? neutralPerspective(),
    userEditedCrop: baseEdits.geometry.crop !== null
  };
  void loadPinnedPreview(ctx, baseEdits, sessionId);
}

export async function finishSession(ctx: GeometryCtx): Promise<void> {
  const sess = ctx.geometrySession;
  if (!sess) return;
  if (sess.pinnedUrl) revoke(sess.pinnedUrl);
  ctx.geometrySession = null;
  const dc = sess.draftCrop;
  const full = dc.x === 0 && dc.y === 0 && dc.w === 1 && dc.h === 1;
  const geometry = {
    ...ctx.edits.geometry,
    rotate: sess.draftRotate,
    flip_h: sess.draftFlipH,
    flip_v: sess.draftFlipV,
    rotate_angle: sess.draftAngle,
    crop: full ? null : sess.draftCrop,
    aspect: sess.draftAspect,
    perspective: perspectiveIsIdentity(sess.draftPerspective)
      ? null
      : clampPerspective(sess.draftPerspective)
  };
  if (JSON.stringify(ctx.edits.geometry) === JSON.stringify(geometry)) return;
  ctx.edits = { ...ctx.edits, geometry };
  await ctx.onCommit('Geometry');
}

export function rotateStep(ctx: GeometryCtx, delta: 90 | 270): void {
  const sess = ctx.geometrySession;
  if (!sess) {
    ctx.edits.geometry.rotate = ((ctx.edits.geometry.rotate + delta) % 360) as 0 | 90 | 180 | 270;
    void ctx.onCommit('Rotate');
    return;
  }
  sess.draftRotate = ((sess.draftRotate + delta) % 360) as 0 | 90 | 180 | 270;
  const { sw, sh } = sourceDims(sess);
  const ratio = aspectRatioFor(sess.draftAspect, sw, sh);
  sess.draftCrop =
    ratio !== null
      ? largestInscribedRect(sw, sh, sess.draftAngle, ratio, perspInv(sess))
      : FULL_CROP;
  sess.userEditedCrop = false;
}

export function flipStep(ctx: GeometryCtx, axis: 'h' | 'v'): void {
  const sess = ctx.geometrySession;
  if (!sess) {
    if (axis === 'h') ctx.edits.geometry.flip_h = !ctx.edits.geometry.flip_h;
    else ctx.edits.geometry.flip_v = !ctx.edits.geometry.flip_v;
    void ctx.onCommit('Flip');
    return;
  }
  if (axis === 'h') sess.draftFlipH = !sess.draftFlipH;
  else sess.draftFlipV = !sess.draftFlipV;
}

export function updateDraftAngle(ctx: GeometryCtx, angle: number): void {
  const sess = ctx.geometrySession;
  if (!sess) return;
  sess.draftAngle = angle;
  refitDraftCrop(sess);
}

export function updateDraftPerspective(ctx: GeometryCtx, patch: Partial<PerspectiveEdits>): void {
  const sess = ctx.geometrySession;
  if (!sess) return;
  sess.draftPerspective = limitPerspective(
    sess.draftPerspective,
    clampPerspective({ ...sess.draftPerspective, ...patch })
  );
  refitDraftCrop(sess);
}

export function updateDraftCrop(ctx: GeometryCtx, crop: CropRect): void {
  const sess = ctx.geometrySession;
  if (!sess) return;
  const { sw, sh } = sourceDims(sess);
  sess.draftCrop = constrainCropRect(crop, sess.draftCrop, sw, sh, sess.draftAngle, perspInv(sess));
  sess.userEditedCrop = true;
}

export function updateDraftAspect(ctx: GeometryCtx, aspect: AspectLock): void {
  const sess = ctx.geometrySession;
  if (!sess) return;
  const { sw, sh } = sourceDims(sess);
  sess.draftAspect = aspect;
  const ratio = aspectRatioFor(aspect, sw, sh);
  if (ratio === null) return;
  sess.draftCrop = largestInscribedRect(sw, sh, sess.draftAngle, ratio, perspInv(sess));
  sess.userEditedCrop = false;
}

export function resetDraft(ctx: GeometryCtx): void {
  const sess = ctx.geometrySession;
  if (!sess) return;
  sess.draftAngle = 0;
  sess.draftAspect = { kind: 'original' };
  sess.draftCrop = FULL_CROP;
  sess.draftPerspective = neutralPerspective();
  sess.userEditedCrop = false;
}
