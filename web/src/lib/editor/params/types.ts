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
  subgroup?: string;
  defaultValue: number;
  rawDefaultValue?: number;
  inactive?: (edits: Edits, isRaw: boolean) => boolean;
  previewMode?: PreviewMode;
  liveGradient?: (edits: Edits) => string;
}

export interface SliderRange extends ParamRange {
  defaultValue?: number;
}

export interface ToolRange extends ParamRange {
  defaultValue: number;
  keyStep: number;
}

export interface SectionDef {
  id: DevelopSection;
  title: string;
  history?: string;
  synonyms?: readonly string[];
  rows: readonly (readonly ParamDef[])[];
  clear?: (edits: Edits) => void;
}

export type DesktopPanelId =
  | 'dcp'
  | 'basic'
  | 'curves'
  | 'hsl'
  | 'color-grading'
  | 'bw'
  | 'lut'
  | 'detail'
  | 'lens'
  | 'lens-blur'
  | 'effects';

export interface DesktopPanel {
  id: DesktopPanelId;
  title: string;
  sections: readonly SectionDef[];
}

export interface TouchView {
  id: string;
  label: string;
  params: readonly ParamDef[];
  sections?: readonly SectionDef[];
}

export type TouchGroupId = 'light' | 'color' | 'effects' | 'detail' | 'optics' | 'profile';

export interface TouchGroup {
  id: TouchGroupId;
  label: string;
  views: readonly TouchView[];
}
