import ParamSection from '#lib/components/editor/controls/ParamSection.svelte';
import {
  PRESENCE,
  TONE,
  clarity,
  dehaze,
  saturation,
  texture,
  vibrance
} from '#lib/editor/params/basic.ts';
import type { TouchGroupId } from '#lib/editor/params/types.ts';
import type { Component } from 'svelte';
import WhiteBalanceSection from './basic/WhiteBalanceSection.svelte';
import BwPanel from './Bw.svelte';
import ColorGradingPanel from './ColorGrading.svelte';
import CurvesPanel from './Curves.svelte';
import DcpPanel from './Dcp.svelte';
import NoiseReductionSection from './detail/NoiseReductionSection.svelte';
import SharpeningSection from './detail/SharpeningSection.svelte';
import EffectsPanel from './Effects.svelte';
import HslPanel from './Hsl.svelte';
import LensPanel from './Lens.svelte';
import LensBlurPanel from './LensBlur.svelte';
import LutPanel from './Lut.svelte';

export type ViewPart = { component: Component<Record<string, unknown>>; props: object };

export type TouchViewKey = `${TouchGroupId}/${string}`;

function part<P extends Record<string, unknown>>(component: Component<P>, props: P): ViewPart {
  return { component: component as unknown as ViewPart['component'], props };
}

function only(component: Component): ViewPart {
  return { component, props: {} };
}

export const TOUCH_VIEW_PARTS: Readonly<Record<TouchViewKey, readonly ViewPart[]>> = {
  'light/tone': [part(ParamSection, { section: TONE })],
  'light/curve': [only(CurvesPanel)],
  'color/basic': [
    only(WhiteBalanceSection),
    part(ParamSection, { section: PRESENCE, params: [vibrance, saturation] })
  ],
  'color/mix': [only(HslPanel)],
  'color/grading': [only(ColorGradingPanel)],
  'color/bw': [only(BwPanel)],
  'effects/basic': [
    part(ParamSection, { section: PRESENCE, params: [texture, clarity, dehaze] }),
    only(EffectsPanel)
  ],
  'effects/lens_blur': [only(LensBlurPanel)],
  'detail/sharpening': [only(SharpeningSection)],
  'detail/noise': [only(NoiseReductionSection)],
  'optics/corrections': [only(LensPanel)],
  'profile/camera': [only(DcpPanel)],
  'profile/lut': [only(LutPanel)]
};

export function touchViewKey(group: TouchGroupId, view: string): TouchViewKey {
  return `${group}/${view}`;
}

export function touchViewParts(group: TouchGroupId, view: string): readonly ViewPart[] {
  return TOUCH_VIEW_PARTS[touchViewKey(group, view)] ?? [];
}
