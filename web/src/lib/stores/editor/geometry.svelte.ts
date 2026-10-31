import {
  FULL_CROP,
  neutralGeometry,
  type AspectLock,
  type CropRect,
  type Edits,
  type GeometryEdits
} from '#lib/types/edits.ts';
import { livePreview } from '#lib/api/preview.ts';
import { keysFor } from '#lib/shortcuts/labels.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { geometryIsIdentity } from '#lib/types/edits.ts';
import { errorMessage } from '#lib/utils/errors.ts';
import { makeObjectUrl, revoke } from '#lib/utils/objectUrl.ts';
import {
  aspectRatioFor,
  clampStraighten,
  constrainCropRect,
  largestInscribedRect,
  quarterTurnCrop,
  refitCropAtAspect
} from '#lib/utils/geom.ts';
import {
  clampPerspective,
  limitPerspective,
  neutralPerspective,
  perspectiveInverse,
  perspectiveIsIdentity,
  type Mat3,
  type PerspectiveEdits
} from '#lib/utils/perspective.ts';
import { LIVE_EDGE } from './previewEdge';
import type { UprightGuide } from '#lib/utils/upright.ts';

export interface GeometrySession {
  frameUrl: string | null;
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
  guides: UprightGuide[];
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

const WHEEL_COMMIT_MS = 300;
const CROP_EPS = 1e-3;

let frameKey: string | null = null;
let frameAbort: AbortController | null = null;
let wheelTimer: ReturnType<typeof setTimeout> | null = null;

export function ready(ctx: GeometryCtx): GeometrySession | null {
  const sess = ctx.geometrySession;
  return sess && sess.srcW > 0 ? sess : null;
}

function perspInv(sess: GeometrySession): Mat3 {
  return perspectiveInverse(sess.draftPerspective);
}

export function sourceDims(sess: GeometrySession): { sw: number; sh: number } {
  const swapped = sess.draftRotate === 90 || sess.draftRotate === 270;
  return { sw: swapped ? sess.srcH : sess.srcW, sh: swapped ? sess.srcW : sess.srcH };
}

export function cropIsDefault(sess: GeometrySession): boolean {
  if (sess.srcW <= 0) return !sess.userEditedCrop;
  const { sw, sh } = sourceDims(sess);
  const fit = largestInscribedRect(sw, sh, sess.draftAngle, sw / sh, perspInv(sess));
  const crop = sess.draftCrop;
  return (
    Math.abs(crop.x - fit.x) < CROP_EPS &&
    Math.abs(crop.y - fit.y) < CROP_EPS &&
    Math.abs(crop.w - fit.w) < CROP_EPS &&
    Math.abs(crop.h - fit.h) < CROP_EPS
  );
}

export function refitDraftCrop(sess: GeometrySession): void {
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

async function loadFrame(
  ctx: GeometryCtx,
  sess: GeometrySession,
  assetId: string,
  edits: Edits,
  controller: AbortController
): Promise<void> {
  let url: string | null = null;
  try {
    const { blob } = await livePreview(
      assetId,
      edits,
      LIVE_EDGE,
      'none',
      undefined,
      controller.signal,
      'geometry'
    );
    url = makeObjectUrl(blob);
    const dims = await new Promise<{ w: number; h: number }>((resolve, reject) => {
      const img = new Image();
      img.onload = () => resolve({ w: img.naturalWidth, h: img.naturalHeight });
      img.onerror = () => reject(new Error('geometry frame decode failed'));
      img.src = url as string;
    });
    if (controller.signal.aborted || ctx.geometrySession !== sess || dims.w <= 0 || dims.h <= 0) {
      revoke(url);
      return;
    }
    if (sess.frameUrl) revoke(sess.frameUrl);
    sess.frameUrl = url;
    sess.srcW = dims.w;
    sess.srcH = dims.h;
  } catch (e) {
    if (url) revoke(url);
    if (controller.signal.aborted || ctx.geometrySession !== sess) return;
    frameKey = null;
    ctx.error = errorMessage(e);
  }
}

export function refreshFrame(ctx: GeometryCtx, edits: Edits): void {
  const sess = ctx.geometrySession;
  if (!sess || !ctx.assetId) return;
  const frame: Edits = { ...edits, geometry: neutralGeometry() };
  const key = `${ctx.assetId}:${JSON.stringify(frame)}`;
  if (key === frameKey) return;
  frameKey = key;
  frameAbort?.abort();
  const controller = new AbortController();
  frameAbort = controller;
  void loadFrame(ctx, sess, ctx.assetId, frame, controller);
}

function dropGuidesIfTurned(sess: GeometrySession, geometry: GeometryEdits): void {
  const turned =
    sess.draftRotate !== geometry.rotate ||
    sess.draftFlipH !== geometry.flip_h ||
    sess.draftFlipV !== geometry.flip_v;
  if (turned) sess.guides = [];
}

function dropGuides(ctx: GeometryCtx): void {
  if (ctx.geometrySession) ctx.geometrySession.guides = [];
}

function resetDrafts(sess: GeometrySession, geometry: GeometryEdits): void {
  dropGuidesIfTurned(sess, geometry);
  sess.draftRotate = geometry.rotate;
  sess.draftFlipH = geometry.flip_h;
  sess.draftFlipV = geometry.flip_v;
  sess.draftAngle = geometry.rotate_angle;
  sess.draftCrop = geometry.crop ?? FULL_CROP;
  sess.draftAspect = geometry.aspect;
  sess.draftPerspective = geometry.perspective ?? neutralPerspective();
  sess.userEditedCrop = geometry.crop !== null;
}

export function startSession(ctx: GeometryCtx): void {
  if (!ctx.assetId || !ctx.initialised || ctx.geometrySession) return;
  ctx.clearView();
  const edits = $state.snapshot(ctx.edits) as Edits;
  const sess: GeometrySession = {
    frameUrl: null,
    srcW: 0,
    srcH: 0,
    draftRotate: 0,
    draftFlipH: false,
    draftFlipV: false,
    draftAngle: 0,
    draftCrop: FULL_CROP,
    draftAspect: { kind: 'original' },
    draftPerspective: neutralPerspective(),
    userEditedCrop: false,
    guides: []
  };
  resetDrafts(sess, edits.geometry);
  ctx.geometrySession = sess;
  refreshFrame(ctx, edits);
}

function draftGeometry(sess: GeometrySession, base: GeometryEdits): GeometryEdits {
  const dc = sess.draftCrop;
  const full = dc.x === 0 && dc.y === 0 && dc.w === 1 && dc.h === 1;
  return {
    ...base,
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
}

function sameGeometry(a: GeometryEdits, b: GeometryEdits): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}

export function syncSession(ctx: GeometryCtx, geometry: GeometryEdits): void {
  const sess = ctx.geometrySession;
  if (!sess || sameGeometry(draftGeometry(sess, geometry), geometry)) return;
  resetDrafts(sess, geometry);
}

export async function commit(
  ctx: GeometryCtx,
  action: string,
  sess = ctx.geometrySession
): Promise<void> {
  if (!sess) return;
  const geometry = draftGeometry(sess, ctx.edits.geometry);
  if (sameGeometry(ctx.edits.geometry, geometry)) return;
  dropGuidesIfTurned(sess, ctx.edits.geometry);
  ctx.edits = { ...ctx.edits, geometry };
  await ctx.onCommit(action);
}

export function scheduleCropCommit(ctx: GeometryCtx): void {
  if (wheelTimer) clearTimeout(wheelTimer);
  wheelTimer = setTimeout(() => {
    wheelTimer = null;
    void commit(ctx, 'Crop');
  }, WHEEL_COMMIT_MS);
}

export async function endSession(ctx: GeometryCtx): Promise<void> {
  const sess = ctx.geometrySession;
  if (!sess) return;
  const pending = wheelTimer !== null;
  if (wheelTimer) clearTimeout(wheelTimer);
  wheelTimer = null;
  frameAbort?.abort();
  frameAbort = null;
  frameKey = null;
  if (sess.frameUrl) revoke(sess.frameUrl);
  ctx.geometrySession = null;
  if (pending) await commit(ctx, 'Crop', sess);
}

function turnedAspect(aspect: AspectLock): AspectLock {
  return aspect.kind === 'ratio' ? { kind: 'ratio', num: aspect.den, den: aspect.num } : aspect;
}

export function rotateStep(ctx: GeometryCtx, delta: 90 | 270): void {
  const sess = ready(ctx);
  if (!sess) {
    const g = ctx.edits.geometry;
    const clockwise = (delta === 90) !== (g.flip_h !== g.flip_v);
    g.rotate = ((g.rotate + delta) % 360) as 0 | 90 | 180 | 270;
    if (g.crop) g.crop = quarterTurnCrop(g.crop, clockwise);
    g.aspect = turnedAspect(g.aspect);
    void ctx.onCommit('Rotate');
    return;
  }
  const clockwise = (delta === 90) !== (sess.draftFlipH !== sess.draftFlipV);
  sess.draftRotate = ((sess.draftRotate + delta) % 360) as 0 | 90 | 180 | 270;
  sess.draftAspect = turnedAspect(sess.draftAspect);
  const { sw, sh } = sourceDims(sess);
  const turned = quarterTurnCrop(sess.draftCrop, clockwise);
  sess.draftCrop = constrainCropRect(turned, turned, sw, sh, sess.draftAngle, perspInv(sess));
  void commit(ctx, 'Rotate');
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
  void commit(ctx, 'Flip');
}

export function updateDraftAngle(ctx: GeometryCtx, angle: number): void {
  const sess = ready(ctx);
  if (!sess) return;
  sess.draftAngle = clampStraighten(angle);
  refitDraftCrop(sess);
}

export function updateDraftPerspective(ctx: GeometryCtx, patch: Partial<PerspectiveEdits>): void {
  const sess = ready(ctx);
  if (!sess) return;
  sess.draftPerspective = limitPerspective(
    sess.draftPerspective,
    clampPerspective({ ...sess.draftPerspective, ...patch })
  );
  refitDraftCrop(sess);
}

export function updateDraftCrop(ctx: GeometryCtx, crop: CropRect): void {
  const sess = ready(ctx);
  if (!sess) return;
  const { sw, sh } = sourceDims(sess);
  sess.draftCrop = constrainCropRect(crop, sess.draftCrop, sw, sh, sess.draftAngle, perspInv(sess));
  sess.userEditedCrop = !cropIsDefault(sess);
}

export async function resetTransform(ctx: GeometryCtx): Promise<void> {
  dropGuides(ctx);
  updateDraftAngle(ctx, 0);
  updateDraftPerspective(ctx, neutralPerspective());
  await commit(ctx, 'Reset transform');
}

export async function reset(ctx: GeometryCtx): Promise<void> {
  dropGuides(ctx);
  if (!ctx.assetId || geometryIsIdentity(ctx.edits.geometry)) return;
  if (wheelTimer) clearTimeout(wheelTimer);
  wheelTimer = null;
  ctx.edits = { ...ctx.edits, geometry: neutralGeometry() };
  if (ctx.geometrySession) resetDrafts(ctx.geometrySession, ctx.edits.geometry);
  await ctx.onCommit('Reset Geometry');
  toasts.push('success', `Reset geometry. Undo with ${keysFor('undo')}.`, 3000);
}

export async function flipAspect(ctx: GeometryCtx): Promise<void> {
  const aspect = ctx.geometrySession?.draftAspect;
  if (aspect?.kind !== 'ratio' || aspect.num === aspect.den) return;
  await setAspect(ctx, { kind: 'ratio', num: aspect.den, den: aspect.num });
}

export async function setAspect(
  ctx: GeometryCtx,
  aspect: AspectLock,
  action = 'Aspect ratio'
): Promise<void> {
  const sess = ready(ctx);
  if (!sess) return;
  const { sw, sh } = sourceDims(sess);
  sess.draftAspect = aspect;
  const ratio = aspectRatioFor(aspect, sw, sh);
  if (ratio !== null) {
    sess.draftCrop = largestInscribedRect(sw, sh, sess.draftAngle, ratio, perspInv(sess));
    sess.userEditedCrop = false;
  }
  await commit(ctx, action);
}
