import { getJson, sendJson } from './client';

export interface Preferences {
  pinnedSettings: string[] | null;
}

interface PreferencesBody {
  pinned_settings: unknown;
}

function pinnedFrom(value: unknown): string[] | null {
  if (!Array.isArray(value)) return null;
  return value.filter((id): id is string => typeof id === 'string');
}

function fromBody(body: PreferencesBody): Preferences {
  return {
    pinnedSettings: pinnedFrom(body.pinned_settings)
  };
}

export async function getPreferences(): Promise<Preferences> {
  return fromBody(await getJson<PreferencesBody>('/api/preferences', undefined, { silent: true }));
}

export async function patchPreferences(patch: Partial<Preferences>): Promise<Preferences> {
  const body: Record<string, unknown> = {};
  if (patch.pinnedSettings !== undefined) body.pinned_settings = patch.pinnedSettings;
  return fromBody(await sendJson<PreferencesBody>('PATCH', '/api/preferences', body));
}
