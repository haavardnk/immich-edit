import { neutralColor } from './color';
import { NEUTRAL_DETAIL } from './detail';
import { NEUTRAL_EFFECTS } from './effects';
import { neutralGeometry } from './geometry';
import { NEUTRAL_LENS } from './lens';
import { NEUTRAL_LENS_BLUR, resetLensBlur } from './lensBlur';
import type { Edits } from './model';
import { neutralBasic, neutralTone } from './tone';

export function neutralEdits(): Edits {
  return {
    basic: neutralBasic(),
    tone: neutralTone(),
    color: neutralColor(),
    detail: { ...NEUTRAL_DETAIL },
    effects: { ...NEUTRAL_EFFECTS },
    lens: { ...NEUTRAL_LENS },
    geometry: neutralGeometry(),
    masks: [],
    retouch: [],
    lens_blur: { ...NEUTRAL_LENS_BLUR }
  };
}

export function resetDevelopEdits(edits: Edits): Edits {
  const neutral = neutralEdits();
  return {
    ...neutral,
    color: {
      ...neutral.color,
      dcp: { ...edits.color.dcp }
    },
    geometry: edits.geometry,
    masks: edits.masks,
    retouch: edits.retouch,
    lens_blur: resetLensBlur(edits.lens_blur)
  };
}

export function originalPreviewEdits(edits: Edits): Edits {
  const neutral = neutralEdits();
  return {
    ...edits,
    basic: neutral.basic,
    tone: neutral.tone,
    color: { ...neutral.color, dcp: edits.color.dcp },
    detail: neutral.detail,
    effects: neutral.effects,
    masks: [],
    retouch: [],
    lens_blur: resetLensBlur(edits.lens_blur)
  };
}

export type DevelopSection =
  | 'dcp'
  | 'white_balance'
  | 'tone'
  | 'presence'
  | 'curves'
  | 'hsl'
  | 'color_grading'
  | 'bw'
  | 'lut'
  | 'sharpening'
  | 'noise_reduction'
  | 'lens'
  | 'lens_blur'
  | 'vignette'
  | 'grain';
