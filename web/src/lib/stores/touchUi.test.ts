import { afterEach, describe, expect, it, vi } from 'vitest';

const storageKey = 'immich-edit:touchEditor';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.resetModules();
});

function stubStorage(initial?: unknown): Map<string, string> {
  const values = new Map<string, string>();
  if (initial !== undefined) values.set(storageKey, JSON.stringify(initial));
  vi.stubGlobal('localStorage', {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value)
  });
  return values;
}

describe('touch editor ui', () => {
  it('starts on Light Tone at the compact detent', async () => {
    stubStorage();
    const { touchUi } = await import('./touchUi.svelte');
    expect([touchUi.group, touchUi.view, touchUi.detent]).toEqual(['light', 'tone', 'compact']);
    expect(touchUi.filmstrip).toBe(true);
  });

  it('remembers the view per group and keeps one tray size across views', async () => {
    const values = stubStorage();
    const { touchUi } = await import('./touchUi.svelte');
    touchUi.setView('curve');
    expect(touchUi.detent).toBe('compact');
    touchUi.setDetent('expanded');
    touchUi.setGroup('color');
    expect([touchUi.view, touchUi.detent]).toEqual(['basic', 'expanded']);
    touchUi.setGroup('light');
    expect([touchUi.view, touchUi.detent]).toEqual(['curve', 'expanded']);
    expect(JSON.parse(values.get(storageKey) ?? '{}')).toMatchObject({
      group: 'light',
      views: { light: 'curve' },
      detent: 'expanded'
    });
  });

  it('opens search at the current view size', async () => {
    stubStorage();
    const { touchUi } = await import('./touchUi.svelte');
    touchUi.setDetent('expanded');
    touchUi.openSearch();
    expect([touchUi.searching, touchUi.detent]).toEqual([true, 'expanded']);
  });

  it('drops unknown stored groups, views and detents', async () => {
    stubStorage({
      group: 'bogus',
      views: { color: 'mix', light: 'nope' },
      detent: 'huge',
      filmstrip: false
    });
    const { touchUi } = await import('./touchUi.svelte');
    expect(touchUi.group).toBe('light');
    expect(touchUi.view).toBe('tone');
    expect(touchUi.viewOf('color')).toBe('mix');
    expect(touchUi.detent).toBe('compact');
    expect(touchUi.filmstrip).toBe(false);
  });

  it('keeps Quick as a group with one view', async () => {
    stubStorage({ group: 'quick', detent: 'expanded' });
    const { touchUi } = await import('./touchUi.svelte');
    expect([touchUi.group, touchUi.view]).toEqual(['quick', '']);
    expect(touchUi.detent).toBe('expanded');
    touchUi.setView('tone');
    expect(touchUi.views).toEqual({});
  });

  it('starts every editor visit on Light Tone', async () => {
    stubStorage({ group: 'color', views: { light: 'curve', color: 'mix' }, detent: 'expanded' });
    const { touchUi } = await import('./touchUi.svelte');
    touchUi.startDevelop();
    expect([touchUi.group, touchUi.view, touchUi.detent]).toEqual(['light', 'tone', 'expanded']);
    expect(touchUi.viewOf('color')).toBe('mix');
  });
});
