import { resetParams, sectionLabel } from '#lib/editor/params/define.ts';
import { neutraliseSection } from '#lib/editor/params/neutralise.ts';
import type { DesktopPanel, ParamDef, SectionDef } from '#lib/editor/params/types.ts';
import { resetDevelopPanel } from '#lib/panels/panelReset.ts';
import { editor } from '#lib/stores/editor.svelte.ts';

export function commitParamReset(params: readonly ParamDef[], action: string): void {
  resetParams(editor.edits, params, editor.isRaw);
  void editor.onCommit(action);
}

export function commitSectionReset(section: SectionDef): void {
  editor.edits = neutraliseSection(editor.edits, section.id, editor.isRaw);
  void editor.onCommit(`Reset ${sectionLabel(section)}`);
}

export function commitPanelReset(panel: Pick<DesktopPanel, 'id' | 'title'>): void {
  editor.edits = resetDevelopPanel(editor.edits, panel.id, editor.isRaw);
  void editor.onCommit(`Reset ${panel.title}`);
}

export function commitLut(id: string | null): void {
  editor.edits.color.lut_3d.lut_id = id;
  void editor.onCommit(id ? 'Select LUT' : 'Remove LUT');
}
