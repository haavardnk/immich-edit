import { browseView } from '#lib/stores/browseView.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { compare } from '#lib/stores/compare.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { resetLayout } from '#lib/testing/layoutMock.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import Loupe from './Loupe.svelte';

vi.mock('$app/navigation', () => ({
  afterNavigate: (): void => {},
  goto: vi.fn()
}));

vi.mock('$app/state', () => ({
  page: { url: new URL('http://localhost/photos'), params: {} }
}));

vi.mock('#lib/device/layout.svelte.ts', async () => ({
  layout: (await import('#lib/testing/layoutMock.ts')).fakeLayout
}));

type LoupeProps = ComponentProps<typeof Loupe>;

const Host: Component<{ component: Component<LoupeProps>; props: LoupeProps }> = ComponentHost;

function asset(id: string): AssetSummary {
  return {
    id,
    originalFileName: `${id}.jpg`,
    type: 'IMAGE',
    fileCreatedAt: null,
    updatedAt: null,
    checksum: null,
    isFavorite: false,
    exifInfo: null,
    tags: []
  };
}

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function render(wide: boolean): void {
  resetLayout({ mode: 'touch', wide, landscape: wide });
  browseView.setCompact(!wide);
  component = mount(Host, { target, props: { component: Loupe, props: {} } });
  flushSync();
}

function toolbar(): HTMLElement {
  const element = target.querySelector<HTMLElement>('nav[aria-label="Loupe toolbar"]');
  if (!element) throw new Error('no loupe toolbar');
  return element;
}

function cullBar(): HTMLElement {
  const element = target.querySelector<HTMLElement>('nav[aria-label="Photo actions"]');
  if (!element) throw new Error('no cull bar');
  return element;
}

function buttonNames(root: HTMLElement): string[] {
  return [...root.querySelectorAll('button')].map(
    (button) => button.getAttribute('aria-label') ?? button.textContent.replace(/\s+/g, ' ').trim()
  );
}

function filmstrip(): HTMLElement | null {
  return target.querySelector('[data-testid="filmstrip-scroll"]');
}

beforeEach(() => {
  vi.stubGlobal(
    'fetch',
    vi.fn(() => Promise.reject(new Error('offline')))
  );
  browsing.set([asset('a'), asset('b'), asset('c')]);
  browseView.openLoupe('b');
  compare.exit();
  ui.fullscreen = false;
  ui.loupeFilmstripCollapsed = false;
  ui.loupeCompactFilmstrip = false;
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
  browseView.closeLoupe();
  browseView.setCompact(false);
  resetLayout();
  vi.unstubAllGlobals();
});

describe('Loupe on touch', () => {
  it('overlays a phone bar on the photo and stacks the cull bar with Edit', () => {
    render(false);
    expect(toolbar().className).toContain('absolute');
    expect(toolbar().querySelector('h2')?.textContent).toBe('b.jpg');
    expect(toolbar().querySelector('h2')?.hasAttribute('type')).toBe(false);
    expect(buttonNames(toolbar())).toEqual(['Back (Esc)', 'Info (I)', 'More loupe actions']);
    expect(cullBar().dataset.density).toBe('stacked');
    expect(buttonNames(cullBar())).toContain('Edit');
    expect(target.textContent).not.toContain('Auto-advance');
    expect(filmstrip()).toBeNull();
  });

  it('keeps the phone filmstrip choice apart from the tablet one', () => {
    render(false);
    browseView.toggleLoupeFilmstrip();
    flushSync();
    expect(filmstrip()).not.toBeNull();
    expect(ui.loupeFilmstripCollapsed).toBe(false);
  });

  it('gives wide touch a solid bar with Edit and a one-row cull bar over the filmstrip', () => {
    render(true);
    expect(toolbar().className).not.toContain('absolute');
    expect(buttonNames(toolbar())).toEqual([
      'Back (Esc)',
      'Info (I)',
      'Edit',
      'More loupe actions'
    ]);
    expect(cullBar().dataset.density).toBe('row');
    expect(buttonNames(cullBar())).toContain('Collapse filmstrip');
    expect(buttonNames(cullBar())).not.toContain('Edit');
    expect(filmstrip()).not.toBeNull();
    expect(
      cullBar().compareDocumentPosition(filmstrip() ?? cullBar()) & Node.DOCUMENT_POSITION_FOLLOWING
    ).toBeTruthy();
  });
});
