import type { ParamDef } from '#lib/editor/params/types.ts';
import { getContext, setContext } from 'svelte';

export type SliderFocus = { readonly focus: (param: ParamDef) => void };

const KEY = Symbol('sliderFocus');

export function provideSliderFocus(host: SliderFocus): void {
  setContext(KEY, host);
}

export function sliderFocus(): SliderFocus | null {
  return getContext<SliderFocus | undefined>(KEY) ?? null;
}
