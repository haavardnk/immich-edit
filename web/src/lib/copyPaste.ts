import type { Edits } from '$lib/types/edits';

export interface CopySections {
  basic: boolean;
  tone: boolean;
  color: boolean;
  detail: boolean;
  effects: boolean;
  lens: boolean;
  geometry: boolean;
  masks: boolean;
}

export type SectionKey = keyof CopySections;

export const DEVELOP_KEYS: SectionKey[] = [
  'basic',
  'tone',
  'color',
  'detail',
  'lens',
  'effects'
];

export const SECTION_LABELS: Record<SectionKey, string> = {
  basic: 'Basic',
  tone: 'Tone',
  color: 'Color',
  detail: 'Detail',
  lens: 'Lens Corrections',
  effects: 'Effects',
  geometry: 'Geometry & crop',
  masks: 'Masks'
};

export function allSections(value: boolean): CopySections {
  return {
    basic: value,
    tone: value,
    color: value,
    detail: value,
    effects: value,
    lens: value,
    geometry: value,
    masks: value
  };
}

export const ALL_COPY_SECTIONS: CopySections = allSections(true);

export const DEFAULT_COPY_SECTIONS: CopySections = {
  ...allSections(true),
  geometry: false,
  masks: false
};

export function hasSelectedSections(sections: CopySections): boolean {
  return Object.values(sections).some((v) => v);
}

export function allSelected(sections: CopySections, keys: SectionKey[]): boolean {
  return keys.every((k) => sections[k]);
}

export function applyCopySections(current: Edits, incoming: Edits, sections: CopySections): Edits {
  return {
    basic: sections.basic ? incoming.basic : current.basic,
    tone: sections.tone ? incoming.tone : current.tone,
    color: sections.color ? incoming.color : current.color,
    detail: sections.detail ? incoming.detail : current.detail,
    effects: sections.effects ? incoming.effects : current.effects,
    lens: sections.lens ? incoming.lens : current.lens,
    geometry: sections.geometry ? incoming.geometry : current.geometry,
    masks: sections.masks ? incoming.masks : current.masks
  };
}
