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
  it('reads the pins and drops stray entries', async () => {
    stubFetch({ pinned_settings: ['exposure', 3], shortcuts: {} });
    await expect(getPreferences()).resolves.toEqual({ pinnedSettings: ['exposure'] });
  });

  it('treats missing pins as the default', async () => {
    stubFetch({ pinned_settings: null });
    await expect(getPreferences()).resolves.toMatchObject({ pinnedSettings: null });
  });

  it('patches only the pins, an empty list included', async () => {
    const fetch = stubFetch({ pinned_settings: [] });
    await patchPreferences({ pinnedSettings: [] });
    expect(fetch.sent()).toEqual({ method: 'PATCH', body: { pinned_settings: [] } });
  });
});
