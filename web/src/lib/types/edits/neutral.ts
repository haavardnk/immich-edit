import { neutralColor } from './color';
import { NEUTRAL_DETAIL } from './detail';
import { NEUTRAL_EFFECTS } from './effects';
import { neutralGeometry } from './geometry';
import { NEUTRAL_LENS } from './lens';
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
    retouch: []
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
    retouch: edits.retouch
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
    retouch: []
  };
}

export type DevelopSection =
  | 'white_balance'
  | 'tone'
  | 'presence'
  | 'sharpening'
  | 'noise_reduction'
  | 'vignette'
  | 'grain'
  | 'lens'
  | 'lut'
  | 'bw';

export function neutraliseSection(edits: Edits, section: DevelopSection): Edits {
  const neutral = neutralEdits();
  switch (section) {
    case 'white_balance':
      return {
        ...edits,
        basic: { ...edits.basic, wb_temp: neutral.basic.wb_temp, wb_tint: neutral.basic.wb_tint }
      };
    case 'tone':
      return {
        ...edits,
        basic: {
          ...edits.basic,
          exposure_ev: neutral.basic.exposure_ev,
          brightness: neutral.basic.brightness,
          contrast: neutral.basic.contrast
        },
        tone: neutral.tone
      };
    case 'presence':
      return {
        ...edits,
        basic: {
          ...edits.basic,
          texture: neutral.basic.texture,
          clarity: neutral.basic.clarity,
          dehaze: neutral.basic.dehaze,
          vibrance: neutral.basic.vibrance,
          saturation: neutral.basic.saturation
        }
      };
    case 'sharpening':
      return {
        ...edits,
        detail: {
          ...edits.detail,
          capture_sharpen: neutral.detail.capture_sharpen,
          sharpen_amount: neutral.detail.sharpen_amount,
          sharpen_radius: neutral.detail.sharpen_radius,
          sharpen_detail: neutral.detail.sharpen_detail,
          sharpen_masking: neutral.detail.sharpen_masking
        }
      };
    case 'noise_reduction':
      return {
        ...edits,
        detail: {
          ...edits.detail,
          luma_nr_amount: neutral.detail.luma_nr_amount,
          luma_nr_detail: neutral.detail.luma_nr_detail,
          luma_nr_contrast: neutral.detail.luma_nr_contrast,
          color_nr_amount: neutral.detail.color_nr_amount,
          color_nr_detail: neutral.detail.color_nr_detail,
          color_nr_smoothness: neutral.detail.color_nr_smoothness
        }
      };
    case 'vignette':
      return {
        ...edits,
        effects: {
          ...edits.effects,
          vignette_amount: neutral.effects.vignette_amount,
          vignette_midpoint: neutral.effects.vignette_midpoint,
          vignette_feather: neutral.effects.vignette_feather,
          vignette_roundness: neutral.effects.vignette_roundness
        }
      };
    case 'grain':
      return {
        ...edits,
        effects: {
          ...edits.effects,
          grain_amount: neutral.effects.grain_amount,
          grain_size: neutral.effects.grain_size,
          grain_roughness: neutral.effects.grain_roughness
        }
      };
    case 'lens':
      return { ...edits, lens: neutral.lens };
    case 'lut':
      return { ...edits, color: { ...edits.color, lut_3d: neutral.color.lut_3d } };
    case 'bw':
      return { ...edits, color: { ...edits.color, bw: neutral.color.bw } };
  }
}
