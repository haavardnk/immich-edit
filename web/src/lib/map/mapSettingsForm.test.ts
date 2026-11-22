import { draftOf, draftProblem, sameSettings, settingsOf } from '#lib/map/mapSettingsForm.ts';
import { describe, expect, it } from 'vitest';

const FOLLOW = { styleLight: null, styleDark: null, geocoderUrl: 'https://photon.komoot.io' };

describe('map settings form', () => {
  it('follows Immich until a style URL is set', () => {
    expect(draftOf(FOLLOW).mode).toBe('immich');
    expect(draftOf({ ...FOLLOW, styleDark: 'https://x/dark' })).toMatchObject({
      mode: 'custom',
      light: '',
      dark: 'https://x/dark'
    });
  });

  it('uses one custom URL for both themes and drops URLs when following Immich', () => {
    const draft = { mode: 'custom' as const, light: ' https://x/l ', dark: '', geocoder: ' ' };
    expect(settingsOf(draft)).toEqual({
      styleLight: 'https://x/l',
      styleDark: 'https://x/l',
      geocoderUrl: ''
    });
    expect(settingsOf({ ...draft, mode: 'immich' }).styleLight).toBeNull();
  });

  it('needs a URL in custom mode and compares saved values', () => {
    expect(draftProblem({ mode: 'custom', light: '', dark: '', geocoder: '' })).not.toBeNull();
    expect(draftProblem({ mode: 'immich', light: '', dark: '', geocoder: '' })).toBeNull();
    expect(sameSettings(FOLLOW, { ...FOLLOW })).toBe(true);
    expect(sameSettings(FOLLOW, { ...FOLLOW, geocoderUrl: '' })).toBe(false);
  });
});
