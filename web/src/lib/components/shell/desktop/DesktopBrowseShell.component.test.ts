import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { createRawSnippet, flushSync, mount, unmount, type Component, type Snippet } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import DesktopBrowseShell from './DesktopBrowseShell.svelte';
import { navSidebar } from './navSidebar.svelte.ts';

const route = vi.hoisted(() => ({ page: { url: new URL('http://localhost/photos'), params: {} } }));

const viewport = vi.hoisted(() => {
  const listeners = new Set<(event: Event) => void>();
  const state = { narrow: false };
  window.matchMedia = (query: string): MediaQueryList =>
    ({
      media: query,
      get matches() {
        return state.narrow;
      },
      onchange: null,
      addEventListener: (_type: string, listener: (event: Event) => void) =>
        listeners.add(listener),
      removeEventListener: (_type: string, listener: (event: Event) => void) =>
        listeners.delete(listener),
      addListener: () => {},
      removeListener: () => {},
      dispatchEvent: () => true
    }) as unknown as MediaQueryList;
  return {
    set(narrow: boolean): void {
      state.narrow = narrow;
      for (const listener of [...listeners]) listener(new Event('change'));
    }
  };
});

vi.mock('$app/state', () => route);

vi.mock('$app/navigation', () => ({
  afterNavigate: (): void => {},
  beforeNavigate: (): void => {},
  onNavigate: (): void => {},
  goto: vi.fn()
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
  component = mount(Host, {
    target,
    props: { component: DesktopBrowseShell, props: { children } }
  });
  flushSync();
}

function remount(path: string): void {
  if (component) void unmount(component);
  component = null;
  flushSync();
  render(path);
}

function sidebar(name: string): HTMLElement | null {
  return target.querySelector<HTMLElement>(`aside[aria-label="${name}"]`);
}

function toggle(): void {
  target.querySelector<HTMLButtonElement>('#library-menu')?.click();
  flushSync();
}

beforeEach(() => {
  vi.stubGlobal(
    'fetch',
    vi.fn(() => Promise.reject(new Error('offline')))
  );
  viewport.set(false);
  navSidebar.open = true;
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
  vi.unstubAllGlobals();
});

describe('DesktopBrowseShell', () => {
  it('renders the route inside its main region', () => {
    render('/photos');

    expect(target.querySelector('main [data-testid="route"]')).not.toBeNull();
  });

  it('starts with the library sidebar open on a wide viewport and toggles it', () => {
    render('/photos');
    expect(sidebar('Library')).not.toBeNull();

    toggle();
    expect(sidebar('Library')).toBeNull();

    toggle();
    expect(sidebar('Library')).not.toBeNull();
  });

  it('starts closed on a narrow viewport and opens on toggle', () => {
    viewport.set(true);
    render('/photos');
    expect(sidebar('Library')).toBeNull();

    toggle();
    expect(sidebar('Library')).not.toBeNull();
  });

  it('keeps a wide collapse across an editor visit', () => {
    render('/photos');
    toggle();
    expect(sidebar('Library')).toBeNull();

    remount('/photos');

    expect(sidebar('Library')).toBeNull();
  });

  it('closes the narrow sidebar again when the shell remounts', () => {
    viewport.set(true);
    render('/photos');
    toggle();
    expect(sidebar('Library')).not.toBeNull();

    remount('/photos');

    expect(sidebar('Library')).toBeNull();
  });

  it('follows the viewport when it crosses the breakpoint', () => {
    render('/photos');
    expect(sidebar('Library')).not.toBeNull();

    viewport.set(true);
    flushSync();
    expect(sidebar('Library')).toBeNull();

    viewport.set(false);
    flushSync();
    expect(sidebar('Library')).not.toBeNull();
  });

  it('shows the administration sidebar on settings routes', () => {
    render('/settings');

    expect(sidebar('Administration')).not.toBeNull();
    expect(sidebar('Library')).toBeNull();
  });
});
