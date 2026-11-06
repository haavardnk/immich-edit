import { ADVANCE_MS, ratingHud } from '#lib/browse/ratingHud.svelte.ts';
import { listenForCommands } from '#lib/commands/listen.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { compare } from '#lib/stores/compare.svelte.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import { virtualCopies } from '#lib/stores/virtualCopies.svelte.ts';
import { asset } from '#lib/testing/assets.ts';
import ComponentHost from '#lib/testing/ComponentHost.svelte';
import type { ExifInfo } from '#lib/types/asset.ts';
import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import Loupe from './Loupe.svelte';

vi.mock('$app/navigation', () => ({
  afterNavigate: (): void => {},
  goto: vi.fn()
}));

vi.mock('$app/state', () => ({
  page: { url: new URL('http://localhost/albums/a1'), params: {} }
}));

const copies = vi.hoisted(() => ({ createVirtualCopies: vi.fn() }));

vi.mock('#lib/editor/copies.ts', async (importOriginal) => ({
  ...(await importOriginal<typeof import('#lib/editor/copies.ts')>()),
  createVirtualCopies: copies.createVirtualCopies
}));

type LoupeProps = ComponentProps<typeof Loupe>;

const Host: Component<{ component: Component<LoupeProps>; props: LoupeProps }> = ComponentHost;

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function render(): void {
  component = mount(Host, { target, props: { component: Loupe, props: {} } });
  flushSync();
}

function press(key: string, init: KeyboardEventInit = {}, from: HTMLElement = document.body): void {
  from.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, ...init }));
  flushSync();
}

function query(selector: string): HTMLElement | null {
  return target.querySelector<HTMLElement>(selector);
}

let stopCommands = (): void => {};

beforeEach(() => {
  stopCommands = listenForCommands();
  vi.stubGlobal(
    'fetch',
    vi.fn(() => Promise.reject(new Error('offline')))
  );
  browsing.set([asset('a'), asset('b'), asset('c')]);
  browseView.openLoupe('b');
  compare.exit();
  selection.clear();
  ui.fullscreen = false;
  ui.loupeFilmstripCollapsed = false;
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  stopCommands();
  if (component) void unmount(component);
  component = null;
  target.remove();
  browseView.closeLoupe();
});

describe('Loupe', () => {
  it('shows the focused filename', () => {
    render();
    expect(query('nav[aria-label="Loupe toolbar"] h2')?.textContent).toBe('b.jpg');
  });

  it('compares the photo and the next one, ignoring the grid selection', () => {
    selection.replace(['a', 'c']);
    render();
    press('c');
    expect(compare.members).toEqual(['b', 'c']);
    press('s');
    expect([...selection.selected]).toEqual(['a', 'c']);
  });

  it('leaves fullscreen on Escape before leaving compare, and leaves compare on e', () => {
    render();
    press('c');
    ui.fullscreen = true;
    press('Escape');
    expect(ui.fullscreen).toBe(false);
    expect(compare.mode).toBe('compare');
    ui.fullscreen = true;
    press('e');
    expect(ui.fullscreen).toBe(true);
    expect(compare.mode).toBe('single');
    expect(browseView.loupeId).not.toBeNull();
  });

  it('moves to the next photo with the arrow keys', () => {
    render();
    press('ArrowRight');
    expect(browseView.loupeId).toBe('c');
    expect(query('nav[aria-label="Loupe toolbar"] h2')?.textContent).toBe('c.jpg');
  });

  it('cancels a pending auto-advance when the loupe moves by hand', () => {
    vi.useFakeTimers();
    try {
      render();
      const step = vi.fn();
      ratingHud.show(
        'b',
        'rating',
        { rating: 2, isFavorite: false, rejected: false, label: null, owned: true },
        step
      );
      flushSync();
      expect(query('[data-testid="rating-hud"]')?.textContent).toContain('2 stars');
      press('ArrowRight');
      vi.advanceTimersByTime(ADVANCE_MS * 2);
      expect(step).not.toHaveBeenCalled();
      expect(ratingHud.shown).toBeNull();
      expect(browseView.loupeId).toBe('c');
    } finally {
      vi.useRealTimers();
    }
  });

  it('toggles auto-advance from the toolbar pill', () => {
    browseView.setLoupeAutoAdvance(false);
    render();
    const pill = [...target.querySelectorAll('button')].find(
      (button) => button.textContent?.trim() === 'Auto-advance'
    );
    expect(pill?.getAttribute('aria-pressed')).toBe('false');
    pill?.click();
    flushSync();
    expect(browseView.loupeAutoAdvance).toBe(true);
    expect(pill?.getAttribute('aria-pressed')).toBe('true');
    browseView.setLoupeAutoAdvance(false);
  });

  it('keeps fullscreen on Shift+Escape', () => {
    render();
    ui.fullscreen = true;
    press('Escape', { shiftKey: true });
    expect(ui.fullscreen).toBe(true);
    expect(browseView.loupeId).toBe('b');
  });

  it('leaves keys typed into an editable region alone', () => {
    render();
    const field = document.createElement('div');
    field.contentEditable = 'true';
    Object.defineProperty(field, 'isContentEditable', { value: true });
    target.appendChild(field);
    press('ArrowRight', {}, field);
    press('Escape', {}, field);
    expect(browseView.loupeId).toBe('b');
  });

  it("creates one virtual copy per Mod+' press, ignoring key repeat", async () => {
    copies.createVirtualCopies.mockReset();
    copies.createVirtualCopies.mockResolvedValue(undefined);
    render();
    press("'", { ctrlKey: true });
    await vi.waitFor(() => expect(virtualCopies.busy).toBe(false));
    press("'", { ctrlKey: true, repeat: true });
    expect(copies.createVirtualCopies).toHaveBeenCalledExactlyOnceWith(['b']);
  });

  it('docks the info panel beside the photo and closes it from the toolbar or its close button', () => {
    const taken = '2024-05-06T07:08:09Z';
    browsing.patch('b', {
      localDateTime: '2024-05-06T09:08:09.000Z',
      exifInfo: {
        dateTimeOriginal: taken,
        timeZone: 'UTC+2',
        fileSizeInByte: 2 * 1024 * 1024
      } as ExifInfo
    });
    render();
    const toggle = query('nav[aria-label="Loupe toolbar"] button[aria-label^="Info"]');
    expect(toggle?.getAttribute('aria-pressed')).toBe('false');
    toggle?.click();
    flushSync();

    const info = query('aside[aria-label="Info"]');
    expect(toggle?.getAttribute('aria-pressed')).toBe('true');
    expect(info?.textContent).toContain('b.jpg');
    expect(info?.textContent).toContain(
      new Date('2024-05-06T09:08:09Z').toLocaleString(undefined, { timeZone: 'UTC' })
    );
    expect(info?.textContent).toContain('2.0 MB · JPG');

    query('button[aria-label="Close info"]')?.click();
    flushSync();
    expect(query('aside[aria-label="Info"]')).toBeNull();
    expect(browseView.loupeInfoOpen).toBe(false);
  });
});
