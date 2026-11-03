import { resetParams, sectionLabel } from '#lib/editor/params/define.ts';
import { neutraliseSection } from '#lib/editor/params/neutralise.ts';
import type { ParamDef, SectionDef } from '#lib/editor/params/types.ts';
import { editor } from '#lib/stores/editor.svelte.ts';

export function commitParamReset(params: readonly ParamDef[], action: string): void {
  resetParams(editor.edits, params, editor.isRaw);
  void editor.onCommit(action);
}

export function commitSectionReset(section: SectionDef): void {
  editor.edits = neutraliseSection(editor.edits, section.id, editor.isRaw);
  void editor.onCommit(`Reset ${sectionLabel(section)}`);
}
