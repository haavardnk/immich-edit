import type { DevelopSection, Edits } from '#lib/types/edits.ts';
import { SECTIONS } from './catalog';
import { resetParams } from './define';

function developDraft(edits: Edits): Edits {
  const { geometry, masks, retouch, ...develop } = edits;
  const copy = JSON.parse(JSON.stringify(develop)) as Omit<Edits, 'geometry' | 'masks' | 'retouch'>;
  return { ...copy, geometry, masks, retouch };
}

export function neutraliseSection(edits: Edits, id: DevelopSection, isRaw: boolean): Edits {
  const section = SECTIONS[id];
  const draft = developDraft(edits);
  resetParams(draft, section.rows.flat(), isRaw);
  section.clear?.(draft);
  return draft;
}
