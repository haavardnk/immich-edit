import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { layoutState, resetLayout } from '#lib/testing/layoutMock.ts';
import { leaveLoupeOnEditorPath } from '#lib/browse/loupeActions.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { editedThumbs } from '#lib/stores/editedThumbs.svelte.ts';
import { jobs } from '#lib/stores/jobs.svelte.ts';
import { createRawSnippet, flushSync, mount, unmount, type Component, type Snippet } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import Shell from './Shell.svelte';

const route = vi.hoisted(() => ({ page: { url: new URL('http://localhost/photos'), params: {} } }));

const navigation = vi.hoisted(() => ({
  onNavigate: [] as ((nav: { to: { url: URL } | null }) => void)[]
}));

vi.mock('$app/state', () => route);

vi.mock('#lib/device/layout.svelte.ts', async () => ({
  layout: (await import('#lib/testing/layoutMock.ts')).fakeLayout
}));

vi.mock('$app/navigation', () => ({
  afterNavigate: (): void => {},
  beforeNavigate: (): void => {},
  onNavigate: (callback: (nav: { to: { url: URL } | null }) => void): void => {
    navigation.onNavigate.push(callback);
  },
  goto: vi.fn()
}));

vi.mock('#lib/browse/loupeActions.ts', async (importOriginal) => ({
  ...(await importOriginal<typeof import('#lib/browse/loupeActions.ts')>()),
  leaveLoupeOnEditorPath: vi.fn()
}));

type ShellProps = { children: Snippet };

const Host: Component<{ component: Component<ShellProps>; props: ShellProps }> = ComponentHost;

const children = createRawSnippet(() => ({
  render: () => '<p data-testid="route">route</p>'
}));

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function render(path: string): void {
  route.page.url = new URL(`http://localhost${path}`);
  component = mount(Host, { target, props: { component: Shell, props: { children } } });
  flushSync();
}

function routeContent(): HTMLElement | null {
  return target.querySelector<HTMLElement>('[data-testid="route"]');
}

function routeInert(): boolean {
  for (let node = routeContent(); node; node = node.parentElement) if (node.inert) return true;
  return false;
}

function navigate(path: string | null): void {
  for (const callback of navigation.onNavigate)
    callback({ to: path ? { url: new URL(`http://localhost${path}`) } : null });
}

beforeEach(() => {
  vi.stubGlobal(
    'fetch',
    vi.fn(() => Promise.reject(new Error('offline')))
  );
  vi.spyOn(editedThumbs, 'loadOnce').mockResolvedValue();
  vi.spyOn(jobs, 'load').mockResolvedValue();
  navigation.onNavigate = [];
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  resetLayout();
  browseView.loupeId = null;
  flushSync();
  if (component) void unmount(component);
  component = null;
  target.remove();
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
  vi.mocked(leaveLoupeOnEditorPath).mockClear();
});

describe('Shell', () => {
  it('wraps a browse route in the desktop browse shell', () => {
    render('/photos');

    expect(target.querySelector('[aria-label="Global navigation"]')).not.toBeNull();
    expect(routeContent()?.parentElement?.tagName).toBe('MAIN');
    expect(editedThumbs.loadOnce).toHaveBeenCalled();
    expect(jobs.load).toHaveBeenCalledOnce();
  });

  it('wraps a browse route in the touch shell on a compact layout', () => {
    layoutState.mode = 'touch';
    render('/photos');

    expect(target.querySelector('[aria-label="Global navigation"]')).toBeNull();
    expect(target.querySelector('nav[aria-label="Main"]')).not.toBeNull();
    expect(routeContent()?.parentElement?.tagName).toBe('MAIN');
  });

  it('wraps a browse route in the wide shell on a wide touch layout', () => {
    layoutState.mode = 'touch';
    layoutState.wide = true;
    render('/photos');

    expect(target.querySelector('[aria-label="Global navigation"]')).toBeNull();
    expect(target.querySelector('nav[aria-label="Main"]')).toBeNull();
    expect(target.querySelector('aside[aria-label="Library"]')).not.toBeNull();
    expect(routeContent()?.parentElement?.tagName).toBe('MAIN');
  });

  it('keeps an editor route bare on a compact layout', () => {
    layoutState.mode = 'touch';
    render('/assets/a1');

    expect(target.querySelector('nav[aria-label="Main"]')).toBeNull();
    expect(target.querySelector('main')).toBeNull();
    expect(routeContent()).not.toBeNull();
  });

  it('renders an editor route without the browse shell', () => {
    render('/assets/a1');

    expect(target.querySelector('[aria-label="Global navigation"]')).toBeNull();
    expect(target.querySelector('main')).toBeNull();
    expect(routeContent()).not.toBeNull();
  });

  it('makes the page inert while the loupe is open', () => {
    render('/photos');
    expect(routeInert()).toBe(false);

    browseView.loupeId = 'a1';
    flushSync();
    expect(routeInert()).toBe(true);

    browseView.loupeId = null;
    flushSync();
    expect(routeInert()).toBe(false);
  });

  it('hands every navigation target to the loupe exit check', () => {
    render('/photos');

    navigate('/assets/a1');
    navigate(null);

    expect(leaveLoupeOnEditorPath).toHaveBeenCalledOnce();
    expect(leaveLoupeOnEditorPath).toHaveBeenCalledWith('/assets/a1');
  });
});
