import { neutralEdits, type Edits } from '$lib/types/edits';

type PanelReset = (edits: Edits, neutral: Edits) => Edits;

const PANEL_RESETS: Record<string, PanelReset> = {
  dcp: (edits, neutral) => ({ ...edits, color: { ...edits.color, dcp: neutral.color.dcp } }),
  basic: (edits, neutral) => ({
    ...edits,
    basic: { ...neutral.basic, curves: edits.basic.curves },
    tone: neutral.tone
  }),
  curves: (edits, neutral) => ({
    ...edits,
    basic: { ...edits.basic, curves: neutral.basic.curves }
  }),
  hsl: (edits, neutral) => ({ ...edits, color: { ...edits.color, hsl: neutral.color.hsl } }),
  'color-grading': (edits, neutral) => ({
    ...edits,
    color: { ...edits.color, color_grade: neutral.color.color_grade }
  }),
  bw: (edits, neutral) => ({ ...edits, color: { ...edits.color, bw: neutral.color.bw } }),
  lut: (edits, neutral) => ({ ...edits, color: { ...edits.color, lut_3d: neutral.color.lut_3d } }),
  detail: (edits, neutral) => ({ ...edits, detail: neutral.detail }),
  lens: (edits, neutral) => ({ ...edits, lens: neutral.lens }),
  effects: (edits, neutral) => ({ ...edits, effects: neutral.effects })
};

export function resetDevelopPanel(edits: Edits, panelId: string): Edits {
  const reset = PANEL_RESETS[panelId];
  return reset ? reset(edits, neutralEdits()) : edits;
}
