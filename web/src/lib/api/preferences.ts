import { getJson, sendJson } from './client';
import type { ShortcutMap } from '#lib/shortcuts/custom.ts';

export type ClickTarget = 'editor' | 'loupe';

export interface Preferences {
  clickTarget: ClickTarget;
  pinnedSettings: readonly string[] | null;
  shortcuts: ShortcutMap;
}

interface PreferencesBody {
  click_target: unknown;
  pinned_settings: unknown;
  shortcuts: unknown;
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

function fromBody(body: PreferencesBody): Preferences {
  return {
    clickTarget: body.click_target === 'loupe' ? 'loupe' : 'editor',
    pinnedSettings: strings(body.pinned_settings),
    shortcuts: shortcutsFrom(body.shortcuts)
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
  return fromBody(await sendJson<PreferencesBody>('PATCH', '/api/preferences', body));
}
