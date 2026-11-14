import type { ColorGradeEdits, ColorGradeRegion, Edits } from '#lib/types/edits.ts';
import { param } from './define';
import { BALANCE_GRADIENT, HUE_GRADIENT, luminanceGradient, saturationGradient } from './gradients';
import { AMOUNT, HUE, PERCENT, signedRange } from './range';
import type { ParamAccess, ParamDef, ParamRange, SectionDef } from './types';

export type GradeRegion = 'shadows' | 'midtones' | 'highlights' | 'global';

export const GRADE_REGION_LABELS: Record<GradeRegion, string> = {
  shadows: 'Shadows',
  midtones: 'Midtones',
  highlights: 'Highlights',
  global: 'Global'
};

const REGIONS = Object.keys(GRADE_REGION_LABELS) as GradeRegion[];

function regionOf(edits: Edits, region: GradeRegion): ColorGradeRegion {
  return edits.color.color_grade[region];
}

function satGradient(region: GradeRegion): (edits: Edits) => string {
  return (edits) => saturationGradient(Math.round(regionOf(edits, region).hue));
}

function lumGradient(region: GradeRegion): (edits: Edits) => string {
  return (edits) => {
    const { hue, sat } = regionOf(edits, region);
    return luminanceGradient(Math.round(hue), Math.round(sat));
  };
}

function regionParam(
  region: GradeRegion,
  key: keyof ColorGradeRegion,
  label: string,
  range: ParamRange,
  liveGradient?: (edits: Edits) => string
): ParamDef {
  const name = GRADE_REGION_LABELS[region];
  return param({
    id: `grade_${region}_${key}`,
    label,
    history: `${name} ${label}`,
    synonyms: ['color grading', 'split toning', name.toLowerCase()],
    subgroup: name,
    ...range,
    liveGradient,
    get: (edits) => regionOf(edits, region)[key],
    set: (edits, value) => {
      regionOf(edits, region)[key] = value;
    }
  });
}

function gradeRegion(region: GradeRegion): readonly ParamDef[] {
  return [
    regionParam(region, 'hue', 'Hue', { ...HUE, gradient: HUE_GRADIENT }),
    regionParam(region, 'sat', 'Saturation', AMOUNT, satGradient(region)),
    regionParam(region, 'lum', 'Luminance', signedRange(50), lumGradient(region))
  ];
}

export const GRADE_REGIONS: Record<GradeRegion, readonly ParamDef[]> = Object.fromEntries(
  REGIONS.map((region) => [region, gradeRegion(region)])
) as Record<GradeRegion, readonly ParamDef[]>;

function gradeField(key: keyof Pick<ColorGradeEdits, 'balance' | 'blend'>): ParamAccess {
  return {
    get: (edits) => edits.color.color_grade[key],
    set: (edits, value) => {
      edits.color.color_grade[key] = value;
    }
  };
}

export const gradeBalance = param({
  id: 'grade_balance',
  label: 'Balance',
  history: 'Color Balance',
  synonyms: ['color grading', 'split toning'],
  ...PERCENT,
  gradient: BALANCE_GRADIENT,
  ...gradeField('balance')
});

export const gradeBlend = param({
  id: 'grade_blend',
  label: 'Blending',
  history: 'Color Blending',
  synonyms: ['color grading', 'split toning'],
  ...AMOUNT,
  gradient: 'var(--gradient-grading-blend)',
  ...gradeField('blend')
});

export const GRADING_PARAMS: readonly ParamDef[] = [
  ...REGIONS.flatMap((region) => GRADE_REGIONS[region]),
  gradeBalance,
  gradeBlend
];

export const COLOR_GRADING: SectionDef = {
  id: 'color_grading',
  title: 'Color Grading',
  rows: [...REGIONS.map((region) => GRADE_REGIONS[region]), [gradeBalance, gradeBlend]]
};
