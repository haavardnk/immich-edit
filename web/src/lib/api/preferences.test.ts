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
  it('reads the click target', async () => {
    stubFetch({ click_target: 'loupe', shortcuts: {} });
    await expect(getPreferences()).resolves.toEqual({ clickTarget: 'loupe' });
  });

  it('treats an unknown click target as the editor', async () => {
    stubFetch({ click_target: 'grid' });
    await expect(getPreferences()).resolves.toEqual({ clickTarget: 'editor' });
  });

  it('patches only the click target', async () => {
    const fetch = stubFetch({ click_target: 'loupe' });
    await expect(patchPreferences({ clickTarget: 'loupe' })).resolves.toEqual({
      clickTarget: 'loupe'
    });
    expect(fetch.sent()).toEqual({ method: 'PATCH', body: { click_target: 'loupe' } });
  });
});
