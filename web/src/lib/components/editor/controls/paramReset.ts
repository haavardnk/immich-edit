import { resetParams } from '#lib/editor/params/define.ts';
import type { ParamDef } from '#lib/editor/params/types.ts';
import { editor } from '#lib/stores/editor.svelte.ts';

export function commitParamReset(params: readonly ParamDef[], action: string): void {
  resetParams(editor.edits, params, editor.isRaw);
  void editor.onCommit(action);
}
