import { PARAMS } from '#lib/editor/params/catalog.ts';
import type { ParamDef } from '#lib/editor/params/types.ts';
import { settingOf } from '#lib/editor/settingsSearch.ts';

const PARAM_BY_ID = new Map(PARAMS.map((param) => [param.id, param]));

export function isPinnable(id: string): boolean {
  return PARAM_BY_ID.has(id);
}

export function pinnedParams(pins: readonly string[]): ParamDef[] {
  return pins.flatMap((id) => PARAM_BY_ID.get(id) ?? []);
}

export function quickLabel(param: ParamDef): string {
  return settingOf(param.id)?.label ?? param.label;
}

export function rowCaption(param: ParamDef): string {
  const path = settingOf(param.id)?.touch.path;
  return path ? `${param.label} · ${path}` : param.label;
}
