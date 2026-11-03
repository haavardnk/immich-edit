import type { PreviewMode } from '#lib/api/preview.ts';
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

export interface ParamAccess {
  get: (edits: Edits, isRaw: boolean) => number;
  set: (edits: Edits, value: number, isRaw: boolean) => void;
}

export interface ParamDef extends ParamRange, ParamAccess {
  id: string;
  label: string;
  history: string;
  synonyms: readonly string[];
  defaultValue: number;
  rawDefaultValue?: number;
  inactive?: (edits: Edits, isRaw: boolean) => boolean;
  previewMode?: PreviewMode;
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
