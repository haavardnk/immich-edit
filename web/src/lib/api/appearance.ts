import type { Appearance, ThemeMode } from '#lib/theme/types.ts';
import { getJson, sendJson } from './client';
import { isAppearance } from '#lib/theme/appearance.ts';

interface AppearanceBody {
  mode: ThemeMode;
  light_theme: string;
  dark_theme: string;
  accents: Record<string, string>;
}

function fromBody(body: AppearanceBody): Appearance {
  const appearance: unknown = {
    mode: body.mode,
    lightTheme: body.light_theme,
    darkTheme: body.dark_theme,
    accents: body.accents
  };
  if (!isAppearance(appearance)) throw new Error('unexpected appearance response');
  return appearance;
}

export async function getAppearance(): Promise<Appearance> {
  return fromBody(await getJson<AppearanceBody>('/api/appearance', undefined, { silent: true }));
}

export async function putAppearance(appearance: Appearance): Promise<Appearance> {
  const body: AppearanceBody = {
    mode: appearance.mode,
    light_theme: appearance.lightTheme,
    dark_theme: appearance.darkTheme,
    accents: { ...appearance.accents }
  };
  return fromBody(await sendJson<AppearanceBody>('PUT', '/api/appearance', body));
}
