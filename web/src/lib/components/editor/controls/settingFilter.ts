import type { ParamDef } from '#lib/editor/params/types.ts';
import { getContext, setContext } from 'svelte';

export type SettingFilter = {
  readonly filtering: boolean;
  readonly shows: (id: string) => boolean;
};

const KEY = Symbol('settingFilter');

export function provideSettingFilter(filter: SettingFilter): void {
  setContext(KEY, filter);
}

export function settingFilter(): SettingFilter | null {
  return getContext<SettingFilter | undefined>(KEY) ?? null;
}

export function visibleParams(
  filter: SettingFilter | null,
  params: readonly ParamDef[]
): readonly ParamDef[] {
  return filter ? params.filter((param) => filter.shows(param.id)) : params;
}

export function shownGroup(
  filter: SettingFilter | null,
  groups: readonly (readonly ParamDef[])[],
  active: number
): number {
  const current = groups[active] ?? [];
  if (!filter?.filtering || visibleParams(filter, current).length > 0) return active;
  const first = groups.findIndex((group) => visibleParams(filter, group).length > 0);
  return first >= 0 ? first : active;
}
