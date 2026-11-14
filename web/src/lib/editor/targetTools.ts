import { BW_MIX } from '#lib/editor/params/color.ts';
import { GRADE_REGION_LABELS, GRADE_REGIONS } from '#lib/editor/params/colorGrading.ts';
import { HSL_BAND_PARAMS } from '#lib/editor/params/hsl.ts';
import type { DesktopPanelId, ParamDef, TouchGroupId } from '#lib/editor/params/types.ts';
import type { TargetTool } from '#lib/stores/targetAdjust.svelte.ts';
import {
  HSL_BAND_NAMES,
  type CurveChannel,
  type DevelopSection,
  type Edits
} from '#lib/types/edits.ts';
import { bwChannelAt, gradeRegionAt, hslBandAt, toneAt, type Rgb } from './targetPick';

export interface TargetAxes {
  main: ParamDef;
  hue?: ParamDef;
  lightness?: ParamDef;
}

export type TargetHit =
  | { kind: 'param'; name: string; axes: TargetAxes }
  | { kind: 'curve'; channel: CurveChannel; level: number };

export interface TargetToolDef {
  id: TargetTool;
  label: string;
  panel: DesktopPanelId;
  section: DevelopSection;
  noun: string;
  touch?: { group: TouchGroupId; view: string };
  unavailable?: { when: (edits: Edits) => boolean; reason: string };
  hit: (rgb: Rgb, edits: Edits, channel: CurveChannel) => TargetHit | null;
}

function colorAxes(name: string, params: readonly ParamDef[] | undefined): TargetHit | null {
  const [hue, main, lightness] = params ?? [];
  return main ? { kind: 'param', name, axes: { main, hue, lightness } } : null;
}

export const TARGET_TOOLS: Record<TargetTool, TargetToolDef> = {
  hsl: {
    id: 'hsl',
    label: 'HSL',
    panel: 'hsl',
    section: 'hsl',
    noun: 'band',
    touch: { group: 'color', view: 'mix' },
    hit: (rgb) => {
      const band = hslBandAt(rgb);
      return band === null ? null : colorAxes(HSL_BAND_NAMES[band] ?? '', HSL_BAND_PARAMS[band]);
    }
  },
  curves: {
    id: 'curves',
    label: 'Curves',
    panel: 'curves',
    section: 'curves',
    noun: 'point',
    touch: { group: 'light', view: 'curve' },
    hit: (rgb, _edits, channel) => ({ kind: 'curve', channel, level: toneAt(rgb, channel) })
  },
  bw: {
    id: 'bw',
    label: 'Black & White',
    panel: 'bw',
    section: 'bw',
    noun: 'channel',
    unavailable: {
      when: (edits) => !edits.color.bw.enabled,
      reason: 'Turn on Black & White to adjust it on the photo'
    },
    hit: (rgb) => {
      const channel = bwChannelAt(rgb);
      const main = BW_MIX.find((param) => param.id === `bw_${channel}`);
      return main ? { kind: 'param', name: main.label, axes: { main } } : null;
    }
  },
  grading: {
    id: 'grading',
    label: 'Color Grading',
    panel: 'color-grading',
    section: 'color_grading',
    noun: 'region',
    hit: (rgb, edits) => {
      const { balance, blend } = edits.color.color_grade;
      const region = gradeRegionAt(rgb, balance, blend);
      return colorAxes(GRADE_REGION_LABELS[region], GRADE_REGIONS[region]);
    }
  }
};

export function targetAvailable(tool: TargetTool, edits: Edits): boolean {
  return !TARGET_TOOLS[tool].unavailable?.when(edits);
}

export function targetToolOf(panel: DesktopPanelId): TargetTool | null {
  return Object.values(TARGET_TOOLS).find((tool) => tool.panel === panel)?.id ?? null;
}

export function targetToolOfSection(section: DevelopSection): TargetTool | null {
  return Object.values(TARGET_TOOLS).find((tool) => tool.section === section)?.id ?? null;
}
