import type { SliderScale } from '#lib/components/editor/controls/sliderScale.ts';
import type { DevelopSection, Edits } from '#lib/types/edits.ts';

export interface ParamRange {
  min: number;
  max: number;
  step: number;
  fineStep: number;
  format: (value: number) => string;
  gradient?: string;
  scale?: SliderScale;
}

export interface ParamDef extends ParamRange {
  id: string;
  label: string;
  synonyms: readonly string[];
  defaultValue: number;
  get: (edits: Edits) => number;
  set: (edits: Edits, value: number) => void;
}

export interface SectionDef {
  id: DevelopSection;
  title: string;
  rows: readonly (readonly ParamDef[])[];
}

export interface TouchView {
  id: string;
  label: string;
  params: readonly ParamDef[];
}

export interface TouchGroup {
  id: string;
  label: string;
  views: readonly TouchView[];
}
