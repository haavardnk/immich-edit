import type { CropRatio } from '#lib/api/preferences.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { preferences } from '#lib/stores/preferences.svelte.ts';
import type { AspectLock } from '#lib/types/edits.ts';
import { croppedOutputSize } from '#lib/utils/geom.ts';
import {
  aspectKey,
  isPortraitAspect,
  isPresetRatio,
  longSideFirst,
  orientedAspect,
  parseRatio,
  ratioKey
} from '#lib/panels/geometry/aspects.ts';

export function draftAspect(): AspectLock {
  return editor.geometrySession?.draftAspect ?? editor.edits.geometry.aspect;
}

export function draftPortrait(): boolean {
  return isPortraitAspect(draftAspect());
}

export function draftRatio(): CropRatio | null {
  const aspect = draftAspect();
  return aspect.kind === 'ratio' ? { num: aspect.num, den: aspect.den } : null;
}

export function isPicked(choice: AspectLock['kind'] | CropRatio): boolean {
  const key = typeof choice === 'string' ? choice : ratioKey(choice);
  return aspectKey(draftAspect()) === key;
}

export function pickKind(kind: 'original' | 'free'): void {
  void editor.setGeometryAspect({ kind });
}

export function pickRatio(ratio: CropRatio, portrait = draftPortrait()): void {
  void editor.setGeometryAspect(orientedAspect(ratio, portrait));
}

export function applyTyped(text: string): boolean {
  const ratio = parseRatio(text);
  if (!ratio) return false;
  void editor.setGeometryAspect({ kind: 'ratio', ...ratio });
  return true;
}

export function canFlipOrientation(): boolean {
  const ratio = draftRatio();
  return ratio !== null && ratio.num !== ratio.den;
}

export function setOrientation(portrait: boolean): void {
  if (draftPortrait() === portrait || !canFlipOrientation()) return;
  void editor.flipGeometryAspect();
}

export function savableRatio(): CropRatio | null {
  const ratio = draftRatio();
  return ratio && !isPresetRatio(ratio) ? longSideFirst(ratio) : null;
}

export function isSavedRatio(ratio: CropRatio): boolean {
  const key = ratioKey(ratio);
  return preferences.savedRatios.some((saved) => ratioKey(saved) === key);
}

export function toggleSavedRatio(): void {
  const ratio = savableRatio();
  if (!ratio) return;
  if (isSavedRatio(ratio)) void preferences.removeCropRatio(ratio);
  else void preferences.saveCropRatio(ratio);
}

export function draftOutputSize(): { w: number; h: number } | null {
  const sess = editor.geometrySession;
  if (!sess || !editor.meta) return null;
  return croppedOutputSize(
    {
      ...editor.edits.geometry,
      rotate: sess.draftRotate,
      rotate_angle: sess.draftAngle,
      crop: sess.draftCrop
    },
    editor.meta.source_w,
    editor.meta.source_h
  );
}
