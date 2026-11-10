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

  it('remembers the view per group and the detent per view', async () => {
    const values = stubStorage();
    const { touchUi } = await import('./touchUi.svelte');
    touchUi.setView('curve');
    expect(touchUi.detent).toBe('expanded');
    touchUi.setDetent('peek');
    touchUi.setGroup('color');
    expect([touchUi.view, touchUi.detent]).toEqual(['basic', 'compact']);
    touchUi.setGroup('light');
    expect([touchUi.view, touchUi.detent]).toEqual(['curve', 'peek']);
    expect(JSON.parse(values.get(storageKey) ?? '{}')).toMatchObject({
      group: 'light',
      views: { light: 'curve' },
      detents: { 'light/curve': 'peek' }
    });
  });

  it('raises a peeked view to its last size, else its default, and leaves others alone', async () => {
    const values = stubStorage();
    const { touchUi } = await import('./touchUi.svelte');
    touchUi.setDetent('peek');
    touchUi.raise();
    expect(touchUi.detent).toBe('compact');
    touchUi.setView('curve');
    touchUi.setDetent('peek');
    touchUi.raise();
    expect(touchUi.detent).toBe('expanded');
    touchUi.setView('tone');
    touchUi.setDetent('expanded');
    touchUi.raise();
    expect(touchUi.detent).toBe('expanded');
    touchUi.setDetent('peek');
    touchUi.raise();
    expect(touchUi.detent).toBe('expanded');
    expect(JSON.parse(values.get(storageKey) ?? '{}')).toMatchObject({
      raised: { 'light/tone': 'expanded' }
    });
  });

  it('opens search at the current view size, raising it from peek', async () => {
    stubStorage();
    const { touchUi } = await import('./touchUi.svelte');
    touchUi.setDetent('expanded');
    touchUi.openSearch();
    expect(touchUi.detent).toBe('expanded');
    touchUi.closeSearch();
    touchUi.setDetent('peek');
    touchUi.openSearch();
    expect(touchUi.detent).toBe('expanded');
  });

  it('drops unknown stored groups, views and detents', async () => {
    stubStorage({
      group: 'bogus',
      views: { color: 'mix', light: 'nope' },
      detents: { 'color/mix': 'expanded', 'light/tone': 'huge' },
      filmstrip: false
    });
    const { touchUi } = await import('./touchUi.svelte');
    expect(touchUi.group).toBe('light');
    expect(touchUi.view).toBe('tone');
    expect(touchUi.viewOf('color')).toBe('mix');
    expect(touchUi.detents).toEqual({ 'color/mix': 'expanded' });
    expect(touchUi.filmstrip).toBe(false);
  });

  it('keeps Quick as a group with one view and its own detent', async () => {
    stubStorage({ group: 'quick', detents: { 'quick/': 'expanded' } });
    const { touchUi } = await import('./touchUi.svelte');
    expect([touchUi.group, touchUi.view, touchUi.viewKey]).toEqual(['quick', '', 'quick/']);
    expect(touchUi.detent).toBe('expanded');
    touchUi.setView('tone');
    expect(touchUi.views).toEqual({});
    touchUi.setGroup('light');
    expect(touchUi.detent).toBe('compact');
  });
});
