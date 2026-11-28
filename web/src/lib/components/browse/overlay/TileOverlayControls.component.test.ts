import type { Preferences } from '#lib/api/preferences.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { preferences } from '#lib/stores/preferences.svelte.ts';
import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { resetLayout } from '#lib/testing/layoutMock.ts';
import { flushSync, mount, unmount, type Component } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import TileOverlayMenu from './TileOverlayMenu.svelte';
import TileOverlayPage from './TileOverlayPage.svelte';

vi.mock('#lib/device/layout.svelte.ts', async () => ({
  layout: (await import('#lib/testing/layoutMock.ts')).fakeLayout
}));

const api = vi.hoisted(() => ({
  getPreferences: vi.fn(),
  patchPreferences: vi.fn(async (patch: Partial<Preferences>) => patch as Preferences)
}));

vi.mock('#lib/api/preferences.ts', () => api);

const Host: Component<{ component: Component; props: Record<string, never> }> = ComponentHost;

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function render(view: Component): void {
  component = mount(Host, { target, props: { component: view, props: {} } });
  flushSync();
}

function checkbox(name: string): HTMLButtonElement {
  const box = document.querySelector<HTMLButtonElement>(`[role="checkbox"][aria-label="${name}"]`);
  if (!box) throw new Error(`no checkbox ${name}`);
  return box;
}

function text(): string {
  return document.body.textContent?.replace(/\s+/g, ' ') ?? '';
}

async function settle(): Promise<void> {
  await vi.waitFor(() => expect(api.patchPreferences).toHaveBeenCalled());
  flushSync();
}

beforeEach(() => {
  resetLayout({ mode: 'touch', phone: true });
  target = document.createElement('div');
  document.body.appendChild(target);
  api.patchPreferences.mockClear();
  browseView.setTileInfo('icons');
});

afterEach(async () => {
  if (component) void unmount(component);
  component = null;
  target.remove();
  preferences.shortcuts = {};
  await preferences.setTileOverlay({
    icons: ['edited', 'favorite', 'rejected', 'label', 'rating', 'copy'],
    details: ['filename', 'taken']
  });
  flushSync();
});

describe('Thumbnail info page', () => {
  it('counts the chosen icons and details and hides the shortcut without a keyboard', () => {
    render(TileOverlayPage);

    expect(text()).toContain('Icons 6 of 11');
    expect(text()).toContain('Details 2 of 4');
    expect(checkbox('Edited').getAttribute('aria-checked')).toBe('true');
    expect(checkbox('Albums').getAttribute('aria-checked')).toBe('false');
    expect(text()).not.toContain('cycles the modes');
    expect(
      document.querySelector<HTMLButtonElement>('[aria-label="Reset thumbnail info"]')?.disabled
    ).toBe(true);
  });

  it('saves a toggled icon to the account', async () => {
    render(TileOverlayPage);

    checkbox('Albums').click();
    await settle();

    expect(api.patchPreferences).toHaveBeenLastCalledWith({
      tileOverlay: {
        icons: ['edited', 'favorite', 'rejected', 'label', 'rating', 'copy', 'albums'],
        details: ['filename', 'taken']
      }
    });
    expect(text()).toContain('Icons 7 of 11');
    expect(
      document.querySelector<HTMLButtonElement>('[aria-label="Reset thumbnail info"]')?.disabled
    ).toBe(false);
  });

  it('caps the details at four and explains the cap', async () => {
    await preferences.setTileOverlay({
      icons: [],
      details: ['filename', 'taken', 'place', 'exposure']
    });
    render(TileOverlayPage);

    expect(checkbox('Lens').disabled).toBe(true);
    expect(checkbox('Place').disabled).toBe(false);
    expect(text()).toContain('Up to 4. Turn one off to choose another.');
  });

  it('warns in full mode when the size fits fewer details than chosen', async () => {
    resetLayout();
    browseView.setTileInfo('full');
    browseView.setGridSize('md');
    await preferences.setTileOverlay({
      icons: [],
      details: ['filename', 'taken', 'place', 'exposure']
    });
    render(TileOverlayPage);

    expect(text()).toContain('Showing 2 of 4 at size M. Larger thumbnails show more.');
    expect(text()).toContain('cycles the modes');
  });

  it('drops the shortcut hint when the cycle key is unbound', () => {
    resetLayout();
    preferences.shortcuts = { gridTileInfo: [] };
    render(TileOverlayPage);

    expect(text()).not.toContain('cycles the modes');
  });

  it('resets to the default overlay', async () => {
    await preferences.setTileOverlay({ icons: ['format'], details: ['lens'] });
    render(TileOverlayPage);
    api.patchPreferences.mockClear();

    document.querySelector<HTMLButtonElement>('[aria-label="Reset thumbnail info"]')?.click();
    await settle();

    expect(api.patchPreferences).toHaveBeenLastCalledWith({ tileOverlay: null });
    expect(checkbox('Edited').getAttribute('aria-checked')).toBe('true');
  });
});

describe('Thumbnail info menu', () => {
  it('opens the controls from the toolbar button and names the mode', async () => {
    resetLayout();
    render(TileOverlayMenu);

    const trigger = document.querySelector<HTMLButtonElement>(
      '[aria-label="Thumbnail info: icons"]'
    );
    expect(trigger?.getAttribute('title')).toBeNull();
    trigger?.click();
    await vi.waitFor(() => expect(document.querySelector('[data-popover-content]')).not.toBeNull());

    expect(document.querySelector('[data-popover-content] h2')?.textContent).toBe('Thumbnail info');
    expect(
      document
        .querySelector('[data-popover-content] [role="radiogroup"]')
        ?.getAttribute('aria-label')
    ).toBe('Thumbnail info');
  });
});
