import type { ParamDef } from '#lib/editor/params/types.ts';
import { searchSettings, type SettingMatch } from '#lib/editor/settingsSearch.ts';

class SettingSearchStore {
  query = $state('');
  target = $state.raw<{ id: string } | null>(null);
  matches: SettingMatch[] = $derived(searchSettings(this.query));
  matched = $derived(new Set(this.matches.map((match) => match.setting.id)));
  filtering = $derived(this.query.trim() !== '');
  paletteOpen = $state(false);

  shows = (id: string): boolean => !this.filtering || this.matched.has(id);

  clear = (): void => {
    this.query = '';
  };

  reveal = (id: string): void => {
    this.target = { id };
  };

  revealedIn = (groups: readonly (readonly ParamDef[])[]): number => {
    const id = this.target?.id;
    return id ? groups.findIndex((group) => group.some((param) => param.id === id)) : -1;
  };
}

export const settingSearch = new SettingSearchStore();
