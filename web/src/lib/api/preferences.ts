import { getJson, sendJson } from './client';

export type ClickTarget = 'editor' | 'loupe';

export interface Preferences {
  clickTarget: ClickTarget;
}

interface PreferencesBody {
  click_target: unknown;
}

function fromBody(body: PreferencesBody): Preferences {
  return { clickTarget: body.click_target === 'loupe' ? 'loupe' : 'editor' };
}

export async function getPreferences(): Promise<Preferences> {
  return fromBody(await getJson<PreferencesBody>('/api/preferences', undefined, { silent: true }));
}

export async function patchPreferences(patch: Partial<Preferences>): Promise<Preferences> {
  const body = { click_target: patch.clickTarget };
  return fromBody(await sendJson<PreferencesBody>('PATCH', '/api/preferences', body));
}
