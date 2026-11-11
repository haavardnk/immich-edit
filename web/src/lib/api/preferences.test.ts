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
      pinnedSettings: ['exposure'],
      shortcuts: {},
      desktopSections: null,
      touchSubviews: null
    });
  });

  it('treats an unknown click target as the editor and missing pins as the default', async () => {
    stubFetch({ click_target: 'grid', pinned_settings: null });
    await expect(getPreferences()).resolves.toMatchObject({
      clickTarget: 'editor',
      pinnedSettings: null
    });
  });

  it('patches only the click target', async () => {
    const fetch = stubFetch({ click_target: 'loupe', pinned_settings: null });
    await expect(patchPreferences({ clickTarget: 'loupe' })).resolves.toMatchObject({
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

  it('reads shortcuts and section lists, dropping malformed entries', async () => {
    stubFetch({
      click_target: 'editor',
      pinned_settings: null,
      shortcuts: { undo: ['Mod+y', 4], redo: 'Mod+Shift+y' },
      desktop_sections: [{ id: 'lut', hidden: true }, { hidden: true }, 'basic'],
      touch_subviews: null
    });
    await expect(getPreferences()).resolves.toMatchObject({
      shortcuts: { undo: ['Mod+y'] },
      desktopSections: [{ id: 'lut', hidden: true }],
      touchSubviews: null
    });
  });

  it('patches shortcuts and section lists under their wire names', async () => {
    const fetch = stubFetch({ click_target: 'editor' });
    await patchPreferences({
      shortcuts: { undo: [] },
      desktopSections: null,
      touchSubviews: [{ id: 'light.tone', hidden: true }]
    });
    expect(fetch.sent().body).toEqual({
      shortcuts: { undo: [] },
      desktop_sections: null,
      touch_subviews: [{ id: 'light.tone', hidden: true }]
    });
  });
});
