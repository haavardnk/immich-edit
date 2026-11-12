import { getJson, sendJson } from './client';
import type { SectionEntry } from '#lib/editor/sectionOrder.ts';
import type { ShortcutMap } from '#lib/shortcuts/custom.ts';

export type ClickTarget = 'editor' | 'loupe';

export interface CropRatio {
  num: number;
  den: number;
}

export interface Preferences {
  clickTarget: ClickTarget;
  pinnedSettings: readonly string[] | null;
  shortcuts: ShortcutMap;
  desktopSections: readonly SectionEntry[] | null;
  touchSubviews: readonly SectionEntry[] | null;
  cropRatios: readonly CropRatio[] | null;
}

interface PreferencesBody {
  click_target: unknown;
  pinned_settings: unknown;
  shortcuts: unknown;
  desktop_sections: unknown;
  touch_subviews: unknown;
  crop_ratios: unknown;
}

function strings(value: unknown): string[] | null {
  if (!Array.isArray(value)) return null;
  return value.filter((id): id is string => typeof id === 'string');
}

function shortcutsFrom(value: unknown): ShortcutMap {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return {};
  const map: Record<string, readonly string[]> = {};
  for (const [id, keys] of Object.entries(value)) {
    const list = strings(keys);
    if (list) map[id] = list;
  }
  return map;
}

function sectionsFrom(value: unknown): SectionEntry[] | null {
  if (!Array.isArray(value)) return null;
  return value.flatMap((entry: unknown) => {
    if (!entry || typeof entry !== 'object') return [];
    const { id, hidden } = entry as Record<string, unknown>;
    return typeof id === 'string' ? [{ id, hidden: hidden === true }] : [];
  });
}

function positiveInteger(value: unknown): value is number {
  return typeof value === 'number' && Number.isInteger(value) && value > 0;
}

function cropRatiosFrom(value: unknown): CropRatio[] | null {
  if (!Array.isArray(value)) return null;
  return value.flatMap((entry: unknown) => {
    if (!entry || typeof entry !== 'object') return [];
    const { num, den } = entry as Record<string, unknown>;
    return positiveInteger(num) && positiveInteger(den) ? [{ num, den }] : [];
  });
}

function fromBody(body: PreferencesBody): Preferences {
  return {
    clickTarget: body.click_target === 'loupe' ? 'loupe' : 'editor',
    pinnedSettings: strings(body.pinned_settings),
    shortcuts: shortcutsFrom(body.shortcuts),
    desktopSections: sectionsFrom(body.desktop_sections),
    touchSubviews: sectionsFrom(body.touch_subviews),
    cropRatios: cropRatiosFrom(body.crop_ratios)
  };
}

export async function getPreferences(): Promise<Preferences> {
  return fromBody(await getJson<PreferencesBody>('/api/preferences', undefined, { silent: true }));
}

export async function patchPreferences(patch: Partial<Preferences>): Promise<Preferences> {
  const body: Record<string, unknown> = {};
  if (patch.clickTarget !== undefined) body.click_target = patch.clickTarget;
  if (patch.pinnedSettings !== undefined) body.pinned_settings = patch.pinnedSettings;
  if (patch.shortcuts !== undefined) body.shortcuts = patch.shortcuts;
  if (patch.desktopSections !== undefined) body.desktop_sections = patch.desktopSections;
  if (patch.touchSubviews !== undefined) body.touch_subviews = patch.touchSubviews;
  if (patch.cropRatios !== undefined) body.crop_ratios = patch.cropRatios;
  return fromBody(await sendJson<PreferencesBody>('PATCH', '/api/preferences', body));
}
