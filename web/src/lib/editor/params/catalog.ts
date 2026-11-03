import {
  BASIC_PARAMS,
  PRESENCE,
  TONE,
  blacks,
  brightness,
  clarity,
  contrast,
  dehaze,
  exposure,
  highlights,
  saturation,
  shadows,
  texture,
  vibrance,
  whites
} from './basic';
import type { ParamDef, SectionDef, TouchGroup } from './types';

export const PARAMS: readonly ParamDef[] = [...BASIC_PARAMS];

export const DESKTOP_SECTIONS: readonly SectionDef[] = [TONE, PRESENCE];

export const TOUCH_GROUPS: readonly TouchGroup[] = [
  {
    id: 'light',
    label: 'Light',
    views: [
      {
        id: 'tone',
        label: 'Tone',
        params: [exposure, brightness, contrast, highlights, shadows, whites, blacks]
      }
    ]
  },
  {
    id: 'color',
    label: 'Color',
    views: [{ id: 'basic', label: 'Basic', params: [vibrance, saturation] }]
  },
  {
    id: 'effects',
    label: 'Effects',
    views: [{ id: 'basic', label: 'Basic', params: [texture, clarity, dehaze] }]
  }
];
