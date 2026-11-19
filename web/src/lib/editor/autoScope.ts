import type { DevelopSection, Edits } from '#lib/types/edits.ts';

export type AutoScope = 'all' | 'tone';

export type SectionAutoScope = Exclude<AutoScope, 'all'>;

export const AUTO_ACTIONS: Readonly<Record<AutoScope, string>> = {
  all: 'Auto',
  tone: 'Auto Tone'
};

const SECTION_AUTO: Partial<Record<DevelopSection, SectionAutoScope>> = {
  tone: 'tone'
};

export function sectionAuto(section: DevelopSection): SectionAutoScope | null {
  return SECTION_AUTO[section] ?? null;
}

function toneOf(edits: Edits, suggested: Edits): Edits {
  return {
    ...edits,
    basic: {
      ...edits.basic,
      exposure_ev: suggested.basic.exposure_ev,
      brightness: suggested.basic.brightness,
      contrast: suggested.basic.contrast
    },
    tone: { ...suggested.tone }
  };
}

export function pickAuto(scope: AutoScope, edits: Edits, suggested: Edits): Edits {
  const toned = toneOf(edits, suggested);
  if (scope === 'tone') return toned;
  return {
    ...toned,
    basic: {
      ...toned.basic,
      white_balance: suggested.basic.white_balance,
      dehaze: suggested.basic.dehaze,
      clarity: suggested.basic.clarity,
      vibrance: suggested.basic.vibrance,
      saturation: suggested.basic.saturation
    }
  };
}
