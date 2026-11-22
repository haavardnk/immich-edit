import type { MapSettings } from '#lib/api/map.ts';

export type StyleMode = 'immich' | 'custom';

export interface MapSettingsDraft {
  mode: StyleMode;
  light: string;
  dark: string;
  geocoder: string;
}

export const PHOTON_PUBLIC = 'https://photon.komoot.io';

export function draftOf(settings: MapSettings): MapSettingsDraft {
  const custom = settings.styleLight !== null || settings.styleDark !== null;
  return {
    mode: custom ? 'custom' : 'immich',
    light: settings.styleLight ?? '',
    dark: settings.styleDark ?? '',
    geocoder: settings.geocoderUrl
  };
}

export function settingsOf(draft: MapSettingsDraft): MapSettings {
  const custom = draft.mode === 'custom';
  const light = draft.light.trim();
  const dark = draft.dark.trim();
  return {
    styleLight: custom ? light || dark || null : null,
    styleDark: custom ? dark || light || null : null,
    geocoderUrl: draft.geocoder.trim()
  };
}

export function draftProblem(draft: MapSettingsDraft): string | null {
  if (draft.mode === 'custom' && draft.light.trim() === '' && draft.dark.trim() === '') {
    return 'Enter a style URL or follow Immich';
  }
  return null;
}

export function sameSettings(a: MapSettings, b: MapSettings): boolean {
  return (
    a.styleLight === b.styleLight && a.styleDark === b.styleDark && a.geocoderUrl === b.geocoderUrl
  );
}
