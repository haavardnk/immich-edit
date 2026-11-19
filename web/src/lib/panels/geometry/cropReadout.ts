import type { PillText } from '#lib/editor/pill.ts';
import { aspectText, ratioReadout } from '#lib/panels/geometry/aspects.ts';
import { draftAspect, draftOutputSize } from '#lib/panels/geometry/ratioActions.ts';

function aspectName(): string {
  const aspect = draftAspect();
  if (aspect.kind === 'ratio') return aspectText(aspect);
  return aspect.kind === 'original' ? 'Original' : 'Free';
}

export function cropReadout(): PillText | null {
  const size = draftOutputSize();
  if (!size) return null;
  return { lead: aspectName(), label: `${size.w} × ${size.h} px`, tail: '', mono: true };
}

export function cropDragReadout(): PillText | null {
  const size = draftOutputSize();
  if (!size) return null;
  return {
    lead: '',
    label: `${size.w} × ${size.h}`,
    tail: `· ${ratioReadout({ num: size.w, den: Math.max(size.h, 1) })}`,
    mono: true
  };
}
