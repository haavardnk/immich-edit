import type { DevelopSection } from '#lib/types/edits.ts';
import {
  BASIC_PARAMS,
  CURVES,
  PRESENCE,
  TONE,
  clarity,
  dehaze,
  saturation,
  texture,
  vibrance
} from './basic';
import { BW, COLOR_PARAMS, DCP, LUT } from './color';
import { COLOR_GRADING, GRADING_PARAMS } from './colorGrading';
import { DETAIL_PARAMS, NOISE_REDUCTION, SHARPENING } from './detail';
import { EFFECTS_PARAMS, GRAIN, VIGNETTE } from './effects';
import { HSL, HSL_PARAMS } from './hsl';
import { LENS, LENS_PARAMS } from './lens';
import { LENS_BLUR, LENS_BLUR_PARAMS } from './lensBlur';
import type { DesktopPanel, ParamDef, SectionDef, TouchGroup } from './types';
import { WHITE_BALANCE } from './whiteBalance';

export const PARAMS: readonly ParamDef[] = [
  ...BASIC_PARAMS,
  ...DETAIL_PARAMS,
  ...EFFECTS_PARAMS,
  ...LENS_PARAMS,
  ...LENS_BLUR_PARAMS,
  ...HSL_PARAMS,
  ...GRADING_PARAMS,
  ...COLOR_PARAMS
];

export const SECTIONS: Readonly<Record<DevelopSection, SectionDef>> = {
  dcp: DCP,
  white_balance: WHITE_BALANCE,
  tone: TONE,
  presence: PRESENCE,
  curves: CURVES,
  hsl: HSL,
  color_grading: COLOR_GRADING,
  bw: BW,
  lut: LUT,
  sharpening: SHARPENING,
  noise_reduction: NOISE_REDUCTION,
  lens: LENS,
  lens_blur: LENS_BLUR,
  vignette: VIGNETTE,
  grain: GRAIN
};

export const DESKTOP_PANELS: readonly DesktopPanel[] = [
  { id: 'dcp', title: 'Camera Profile', sections: [DCP] },
  { id: 'basic', title: 'Basic', sections: [WHITE_BALANCE, TONE, PRESENCE] },
  { id: 'curves', title: 'Curves', sections: [CURVES] },
  { id: 'hsl', title: 'HSL', sections: [HSL] },
  { id: 'color-grading', title: 'Color Grading', sections: [COLOR_GRADING] },
  { id: 'bw', title: 'Black & White', sections: [BW] },
  { id: 'lut', title: 'LUT', sections: [LUT] },
  { id: 'detail', title: 'Detail', sections: [SHARPENING, NOISE_REDUCTION] },
  { id: 'lens', title: 'Lens Corrections', sections: [LENS] },
  { id: 'lens-blur', title: 'Lens Blur', sections: [LENS_BLUR] },
  { id: 'effects', title: 'Effects', sections: [VIGNETTE, GRAIN] }
];

export const DESKTOP_SECTIONS: readonly SectionDef[] = DESKTOP_PANELS.flatMap(
  (panel) => panel.sections
);

const flat = (section: SectionDef): readonly ParamDef[] => section.rows.flat();

export const TOUCH_GROUPS: readonly TouchGroup[] = [
  {
    id: 'light',
    label: 'Light',
    views: [
      { id: 'tone', label: 'Tone', params: flat(TONE) },
      { id: 'curve', label: 'Curve', params: [], sections: [CURVES] }
    ]
  },
  {
    id: 'color',
    label: 'Color',
    views: [
      {
        id: 'basic',
        label: 'Basic',
        params: [vibrance, saturation],
        sections: [WHITE_BALANCE]
      },
      { id: 'mix', label: 'Mix', params: flat(HSL) },
      { id: 'grading', label: 'Grading', params: flat(COLOR_GRADING) },
      { id: 'bw', label: 'B&W', params: flat(BW) }
    ]
  },
  {
    id: 'effects',
    label: 'Effects',
    views: [
      {
        id: 'basic',
        label: 'Basic',
        params: [texture, clarity, dehaze, ...flat(VIGNETTE), ...flat(GRAIN)]
      },
      { id: 'lens_blur', label: 'Lens Blur', params: flat(LENS_BLUR) }
    ]
  },
  {
    id: 'detail',
    label: 'Detail',
    views: [
      { id: 'sharpening', label: 'Sharpening', params: flat(SHARPENING) },
      { id: 'noise', label: 'Noise', params: flat(NOISE_REDUCTION) }
    ]
  },
  {
    id: 'optics',
    label: 'Optics',
    views: [{ id: 'corrections', label: 'Corrections', params: flat(LENS) }]
  },
  {
    id: 'profile',
    label: 'Profile',
    views: [
      { id: 'camera', label: 'Camera', params: [], sections: [DCP] },
      { id: 'lut', label: 'LUT', params: flat(LUT) }
    ]
  }
];
