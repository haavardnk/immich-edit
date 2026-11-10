import { afterEach, describe, expect, it, vi } from 'vitest';
import { getPreferences, patchPreferences } from './preferences';

function stubFetch(response: unknown): { sent: () => { method?: string; body: unknown } } {
  let init: RequestInit | undefined;
  vi.stubGlobal(
    'fetch',
    vi.fn(async (_input: RequestInfo, next?: RequestInit) => {
      init = next;
      return new Response(JSON.stringify(response), {
        headers: { 'content-type': 'application/json' }
      });
    })
  );
  return {
    sent: () => ({ method: init?.method, body: init?.body ? JSON.parse(String(init.body)) : null })
  };
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('preferences api', () => {
  it('reads the click target and the pins', async () => {
    stubFetch({ click_target: 'loupe', pinned_settings: ['exposure', 3], shortcuts: {} });
    await expect(getPreferences()).resolves.toEqual({
      clickTarget: 'loupe',
      pinnedSettings: ['exposure']
    });
  });

  it('treats an unknown click target as the editor and missing pins as the default', async () => {
    stubFetch({ click_target: 'grid', pinned_settings: null });
    await expect(getPreferences()).resolves.toEqual({
      clickTarget: 'editor',
      pinnedSettings: null
    });
  });

  it('patches only the click target', async () => {
    const fetch = stubFetch({ click_target: 'loupe', pinned_settings: null });
    await expect(patchPreferences({ clickTarget: 'loupe' })).resolves.toEqual({
      clickTarget: 'loupe',
      pinnedSettings: null
    });
    expect(fetch.sent()).toEqual({ method: 'PATCH', body: { click_target: 'loupe' } });
  });

  it('patches only the pins, an empty list included', async () => {
    const fetch = stubFetch({ click_target: 'editor', pinned_settings: [] });
    await patchPreferences({ pinnedSettings: [] });
    expect(fetch.sent()).toEqual({ method: 'PATCH', body: { pinned_settings: [] } });
  });
});
