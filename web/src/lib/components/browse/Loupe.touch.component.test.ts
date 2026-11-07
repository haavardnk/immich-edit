import { browseView } from '#lib/stores/browseView.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { compare } from '#lib/stores/compare.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { resetLayout } from '#lib/testing/layoutMock.ts';
import { reactiveValue } from '#lib/testing/reactiveValue.svelte.ts';
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

function modeSwitch(): HTMLElement {
  const element = toolbar().querySelector<HTMLElement>('[role="radiogroup"]');
  if (!element) throw new Error('no view mode switch');
  return element;
}

function modeButton(name: string): HTMLButtonElement {
  const button = [...modeSwitch().querySelectorAll('button')].find(
    (candidate) => candidate.textContent.trim() === name
  );
  if (!button) throw new Error(`no ${name} mode`);
  return button;
}

function pressedMode(): string | undefined {
  return modeSwitch().querySelector('[aria-checked="true"]')?.textContent.trim();
}

function paneRail(): HTMLElement {
  const element = target.querySelector<HTMLElement>('[role="toolbar"][aria-label="Pane actions"]');
  if (!element) throw new Error('no pane rail');
  return element;
}

function paneArea(): HTMLElement {
  const element = target.querySelector<HTMLElement>('.grid.bg-neutral-950');
  if (!element) throw new Error('no pane grid');
  return element;
}

function paneBadges(): string[] {
  return [
    ...paneArea().querySelectorAll('[role="button"][tabindex="0"] > div > span:first-child')
  ].map((badge) => badge.textContent.trim());
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

  it('hides the filmstrip and cull bar on a phone while the info sheet is open', () => {
    render(false);
    browseView.toggleLoupeFilmstrip();
    browseView.setLoupeInfoOpen(true);
    flushSync();
    expect(filmstrip()).toBeNull();
    expect(target.querySelector('nav[aria-label="Photo actions"]')).toBeNull();
    browseView.setLoupeInfoOpen(false);
    flushSync();
    expect(filmstrip()).not.toBeNull();
    expect(cullBar().dataset.density).toBe('stacked');
  });

  it('gives wide touch a solid bar with Edit and a one-row cull bar over the filmstrip', () => {
    render(true);
    expect(toolbar().className).not.toContain('absolute');
    expect(buttonNames(toolbar())).toEqual([
      'Back (Esc)',
      'Single',
      'Compare',
      'Survey',
      'Info (I)',
      'Edit',
      'More loupe actions'
    ]);
    expect(modeSwitch().getAttribute('aria-label')).toBe('View mode');
    expect(cullBar().dataset.density).toBe('row');
    expect(buttonNames(cullBar())).toContain('Collapse filmstrip');
    expect(buttonNames(cullBar())).not.toContain('Edit');
    expect(filmstrip()).not.toBeNull();
    expect(
      cullBar().compareDocumentPosition(filmstrip() ?? cullBar()) & Node.DOCUMENT_POSITION_FOLLOWING
    ).toBeTruthy();
  });

  it('switches a phone into compare from the segmented control, which replaces the title', () => {
    render(false);
    compare.enter('survey', ['a', 'b', 'c']);
    flushSync();
    expect(toolbar().querySelector('h2')).toBeNull();
    expect(buttonNames(toolbar())).toEqual([
      'Back (Esc)',
      'Single',
      'Compare',
      'Survey',
      'More loupe actions'
    ]);
    expect(pressedMode()).toBe('Survey');
    modeButton('Compare').click();
    flushSync();
    expect(compare.mode).toBe('compare');
    expect(pressedMode()).toBe('Compare');
    expect(paneArea().getAttribute('style')).toContain('grid-template-columns: repeat(1,');
  });

  it('puts the pane rail between the panes and the cull bar on a phone', () => {
    render(false);
    compare.enter('compare', ['b', 'c'], 1);
    flushSync();
    const rail = paneRail();
    expect(rail.textContent).toContain('c');
    expect(buttonNames(rail)).toEqual(['Sync', 'Promote', 'Drop']);
    expect(rail.compareDocumentPosition(cullBar()) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    [...rail.querySelectorAll('button')]
      .find((button) => button.textContent.trim() === 'Drop')
      ?.click();
    flushSync();
    expect(compare.mode).toBe('single');
    expect(browseView.loupeId).toBe('b');
  });

  it('names the survey pane in the rail and lays a landscape phone side by side', () => {
    resetLayout({ mode: 'touch', wide: false, landscape: true });
    browseView.setCompact(true);
    component = mount(Host, { target, props: { component: Loupe, props: {} } });
    compare.enter('compare', ['a', 'b']);
    flushSync();
    expect(paneArea().getAttribute('style')).toContain('grid-template-columns: repeat(2,');
    compare.enter('survey', ['a', 'b', 'c'], 2);
    flushSync();
    expect(paneRail().textContent).toContain('c');
    expect(paneRail().textContent).not.toContain('3 of 3');
    expect(buttonNames(paneRail())).toContain('Keep');
  });

  it('shows the panes a narrower layout fits around the focused one and keeps the rest', () => {
    const limit = reactiveValue(9);
    const release = compare.limitBy(() => limit.value);
    try {
      browsing.set(['a', 'b', 'c', 'd', 'e', 'f'].map(asset));
      render(true);
      compare.enter('survey', ['a', 'b', 'c', 'd', 'e', 'f'], 5);
      flushSync();
      expect(paneBadges()).toEqual(['1', '2', '3', '4', '5', '6']);
      limit.value = 4;
      flushSync();
      expect(paneBadges()).toEqual(['3', '4', '5', '6']);
      expect(compare.members).toHaveLength(6);
      limit.value = 9;
      flushSync();
      expect(paneBadges()).toEqual(['1', '2', '3', '4', '5', '6']);
    } finally {
      release();
    }
  });

  it('shows the pane actions labelled in the wide cull bar', () => {
    render(true);
    compare.enter('compare', ['b', 'c'], 1);
    flushSync();
    expect(buttonNames(cullBar())).toEqual(expect.arrayContaining(['Sync', 'Promote', 'Drop']));
    expect(target.querySelector('[role="toolbar"][aria-label="Pane actions"]')).toBeNull();
  });
});
