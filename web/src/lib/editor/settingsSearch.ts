import { modifiedSections } from '#lib/editor/modified.ts';
import { DESKTOP_PANELS, TOUCH_GROUPS } from '#lib/editor/params/catalog.ts';
import { paramDefault } from '#lib/editor/params/define.ts';
import type {
  DesktopPanel,
  DesktopPanelId,
  ParamDef,
  SectionDef,
  TouchGroupId
} from '#lib/editor/params/types.ts';
import type { Edits } from '#lib/types/edits.ts';

export interface TouchHome {
  group: TouchGroupId;
  view: string;
  order: number;
  path: string;
}

export interface Setting {
  id: string;
  label: string;
  param: ParamDef | null;
  section: SectionDef;
  panel: DesktopPanelId;
  desktopPath: string;
  touch: TouchHome;
  terms: string;
}

export interface SettingMatch {
  setting: Setting;
  highlight: Highlight | null;
}

export interface SettingGroup {
  home: TouchHome;
  settings: readonly Setting[];
}

const SEPARATOR = ' › ';

function joinPath(parts: readonly (string | undefined)[]): string {
  return parts.filter((part) => !!part).join(SEPARATOR);
}

function touchHomes(): Map<string, TouchHome> {
  const homes = new Map<string, TouchHome>();
  let order = 0;
  for (const group of TOUCH_GROUPS) {
    for (const view of group.views) {
      const viewLabel = group.views.length > 1 ? view.label : undefined;
      const home = (subgroup?: string): TouchHome => ({
        group: group.id,
        view: view.id,
        order,
        path: joinPath([group.label, viewLabel, subgroup])
      });
      for (const param of view.params) homes.set(param.id, home(param.subgroup));
      for (const section of view.sections ?? []) homes.set(sectionKey(section), home());
      order += 1;
    }
  }
  return homes;
}

function sectionKey(section: SectionDef): string {
  return `section:${section.id}`;
}

function sentenceCase(text: string): string {
  return text
    .split(' ')
    .map((word, index) => (index > 0 && /^[A-Z][a-z]+$/.test(word) ? word.toLowerCase() : word))
    .join(' ');
}

function panelPath(panel: DesktopPanel, section: SectionDef): string | undefined {
  return panel.sections.length > 1 ? section.title : undefined;
}

function terms(parts: readonly (string | undefined)[]): string {
  return parts
    .filter((part) => !!part)
    .join('\n')
    .toLowerCase();
}

function buildSettings(): Setting[] {
  const homes = touchHomes();
  const settings: Setting[] = [];
  for (const panel of DESKTOP_PANELS) {
    for (const section of panel.sections) {
      const params = section.rows.flat();
      if (params.length === 0) {
        const touch = homes.get(sectionKey(section));
        if (!touch) continue;
        const desktopPath = section.title === panel.title ? '' : panel.title;
        settings.push({
          id: sectionKey(section),
          label: section.title,
          param: null,
          section,
          panel: panel.id,
          desktopPath,
          touch,
          terms: terms([section.title, ...(section.synonyms ?? []), desktopPath, touch.path])
        });
        continue;
      }
      for (const param of params) {
        const touch = homes.get(param.id);
        if (!touch) continue;
        const label = sentenceCase(param.history);
        const desktopPath = joinPath([panel.title, panelPath(panel, section), param.subgroup]);
        settings.push({
          id: param.id,
          label,
          param,
          section,
          panel: panel.id,
          desktopPath,
          touch,
          terms: terms([label, ...param.synonyms, section.title, desktopPath, touch.path])
        });
      }
    }
  }
  return settings;
}

export const SETTINGS: readonly Setting[] = buildSettings();

export type Highlight = readonly [number, number];

type Ranked = { setting: Setting; highlight: Highlight | null; rank: number };

export function textMatch(text: string, query: string): Highlight | null {
  const needle = query.trim().toLowerCase();
  const at = text.toLowerCase().indexOf(needle);
  return at >= 0 ? [at, at + needle.length] : null;
}

function rank(setting: Setting, needle: string, tokens: readonly string[]): Ranked | null {
  const highlight = textMatch(setting.label, needle);
  if (highlight) {
    const [at] = highlight;
    const wordStart = at === 0 || setting.label[at - 1] === ' ';
    return { setting, highlight, rank: at === 0 ? 0 : wordStart ? 1 : 2 };
  }
  if (tokens.every((token) => setting.terms.includes(token))) {
    return { setting, highlight: null, rank: 3 };
  }
  return null;
}

export function searchSettings(
  query: string,
  settings: readonly Setting[] = SETTINGS
): SettingMatch[] {
  const needle = query.trim().toLowerCase();
  if (!needle) return settings.map((setting) => ({ setting, highlight: null }));
  const tokens = needle.split(/\s+/);
  return settings
    .flatMap((setting) => rank(setting, needle, tokens) ?? [])
    .sort((a, b) => a.rank - b.rank)
    .map(({ setting, highlight }) => ({ setting, highlight }));
}

export function settingValue(setting: Setting, edits: Edits, isRaw: boolean): string {
  const param = setting.param;
  if (!param) return '';
  const value = param.get(edits, isRaw);
  return param.format(Object.is(value, -0) ? 0 : value);
}

export function modifiedSettings(edits: Edits, isRaw: boolean): Setting[] {
  const sections = modifiedSections(edits);
  return SETTINGS.filter((setting) =>
    setting.param
      ? setting.param.get(edits, isRaw) !== paramDefault(setting.param, isRaw)
      : sections.has(setting.section.id)
  );
}

export function byTouchHome(settings: readonly Setting[]): SettingGroup[] {
  const groups = new Map<string, { home: TouchHome; settings: Setting[] }>();
  for (const setting of settings) {
    const key = setting.touch.path;
    const group = groups.get(key);
    if (group) group.settings.push(setting);
    else groups.set(key, { home: setting.touch, settings: [setting] });
  }
  return [...groups.values()].sort((a, b) => a.home.order - b.home.order);
}
