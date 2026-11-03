import { DESKTOP_PANELS } from '#lib/editor/params/catalog.ts';
import { neutraliseSection } from '#lib/editor/params/neutralise.ts';
import type { DesktopPanelId } from '#lib/editor/params/types.ts';
import type { Edits } from '#lib/types/edits.ts';

export function resetDevelopPanel(edits: Edits, panelId: DesktopPanelId, isRaw: boolean): Edits {
  const panel = DESKTOP_PANELS.find(({ id }) => id === panelId);
  if (!panel) return edits;
  return panel.sections.reduce(
    (next, section) => neutraliseSection(next, section.id, isRaw),
    edits
  );
}
