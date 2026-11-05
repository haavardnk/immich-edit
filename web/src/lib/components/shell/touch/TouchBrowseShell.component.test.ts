import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { layoutState, resetLayout } from '#lib/testing/layoutMock.ts';
import { accountDrawer } from '#lib/components/shell/touch/accountDrawer.svelte.ts';
import { touchBars } from '#lib/components/shell/touch/touchBars.svelte.ts';
import { signOutFlow } from '#lib/auth/signOutFlow.svelte.ts';
import { jobs } from '#lib/stores/jobs.svelte.ts';
import { session } from '#lib/stores/session.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import type { Job } from '#lib/api/jobs.ts';
import {
  createRawSnippet,
  flushSync,
  mount,
  tick,
  unmount,
  type Component,
  type Snippet
} from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import TouchBrowseShell from './TouchBrowseShell.svelte';

const route = vi.hoisted(() => ({ page: { url: new URL('http://localhost/photos'), params: {} } }));

const nav = vi.hoisted(() => ({ goto: vi.fn() }));

const auth = vi.hoisted(() => ({ signOut: vi.fn() }));

vi.mock('$app/state', () => route);

vi.mock('$app/navigation', () => ({
  afterNavigate: (): void => {},
  beforeNavigate: (): void => {},
  onNavigate: (): void => {},
  goto: nav.goto
}));

vi.mock('#lib/auth/account.ts', async (importOriginal) => ({
  ...(await importOriginal<typeof import('#lib/auth/account.ts')>()),
  signOut: auth.signOut
}));

vi.mock('#lib/device/layout.svelte.ts', async () => ({
  layout: (await import('#lib/testing/layoutMock.ts')).fakeLayout
}));

type ShellProps = { children: Snippet };

const Host: Component<{ component: Component<ShellProps>; props: ShellProps }> = ComponentHost;

const children = createRawSnippet(() => ({
  render: () =>
    '<div><header data-testid="bar">bar</header><div data-testid="scroller" style="overflow-y:auto">rows</div></div>'
}));

function job(kind: string): Job {
  return {
    id: kind,
    kind,
    status: 'running',
    target: null,
    params: null,
    total: 12,
    completed: 3,
    failed: 0,
    cancelled_at: null,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z'
  };
}

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

async function settle(): Promise<void> {
  flushSync();
  await tick();
  await new Promise((resolve) => setTimeout(resolve, 0));
  flushSync();
}

async function render(path = '/photos'): Promise<void> {
  route.page.url = new URL(`http://localhost${path}`);
  component = mount(Host, { target, props: { component: TouchBrowseShell, props: { children } } });
  await settle();
}

function byLabel(name: string): HTMLElement | null {
  return document.querySelector<HTMLElement>(`[aria-label="${name}"]`);
}

function buttonWithText(text: string): HTMLButtonElement | null {
  return (
    [...document.querySelectorAll<HTMLButtonElement>('button')].find(
      (candidate) => candidate.textContent.trim() === text
    ) ?? null
  );
}

function scrollTo(scroller: HTMLElement, top: number): void {
  Object.defineProperty(scroller, 'scrollTop', { value: top, configurable: true });
  Object.defineProperty(scroller, 'scrollHeight', { value: 5000, configurable: true });
  Object.defineProperty(scroller, 'clientHeight', { value: 400, configurable: true });
  scroller.dispatchEvent(new Event('scroll'));
}

function drawer(): HTMLElement | null {
  return document.querySelector<HTMLElement>('[role="dialog"]');
}

async function openDrawer(): Promise<HTMLElement> {
  accountDrawer.show();
  await settle();
  const element = drawer();
  if (!element) throw new Error('drawer did not open');
  return element;
}

beforeEach(() => {
  vi.spyOn(globalThis, 'fetch').mockRejectedValue(new Error('offline'));
  resetLayout({ mode: 'touch' });
  session.user = {
    id: 'u1',
    email: 'hanne@example.net',
    name: 'Hanne Kristiansen',
    is_admin: true,
    auth_kind: 'oauth'
  };
  jobs.jobs = [];
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(async () => {
  accountDrawer.close();
  signOutFlow.cancel();
  jobs.attention = false;
  jobs.jobs = [];
  jobs.close();
  ui.keybindsHelpOpen = false;
  await settle();
  if (component) void unmount(component);
  component = null;
  target.remove();
  session.user = null;
  vi.restoreAllMocks();
  nav.goto.mockClear();
  auth.signOut.mockClear();
});

describe('TouchBrowseShell', () => {
  it('renders the route in main above a tab bar', async () => {
    await render('/photos');

    expect(target.querySelector('main [data-testid="bar"]')).not.toBeNull();
    const tabs = [...target.querySelectorAll('nav[aria-label="Main"] a')];
    expect(tabs.map((tab) => tab.textContent.trim())).toEqual([
      'Photos',
      'Albums',
      'Search',
      'Library'
    ]);
    expect(tabs.map((tab) => tab.getAttribute('aria-current'))).toEqual(['page', null, null, null]);
  });

  it('marks Library for the collection routes it lists', async () => {
    await render('/people/p1');

    const current = target.querySelector('nav[aria-label="Main"] a[aria-current="page"]');
    expect(current?.textContent.trim()).toBe('Library');
  });

  it('opens the account drawer with the account card and rows', async () => {
    await render();
    const element = await openDrawer();

    expect(element.textContent).toContain('Hanne Kristiansen');
    expect(element.textContent).toContain('hanne@example.net');
    expect(element.textContent).toContain('admin');
    expect(element.textContent).toContain('single sign-on');
    expect(buttonWithText('Settings')).not.toBeNull();
    expect(element.querySelector('[role="radiogroup"][aria-label="Theme"]')).not.toBeNull();
    expect(buttonWithText('Sign out')).not.toBeNull();
    expect(buttonWithText('Keyboard shortcuts')).toBeNull();
  });

  it('shows Keyboard shortcuts once a hardware keyboard was used and opens the help', async () => {
    layoutState.hasKeyboard = true;
    await render();
    await openDrawer();

    [...document.querySelectorAll<HTMLButtonElement>('button')]
      .find((candidate) => candidate.textContent.includes('Keyboard shortcuts'))
      ?.click();
    await settle();

    expect(accountDrawer.open).toBe(false);
    expect(ui.keybindsHelpOpen).toBe(true);
  });

  it('opens Settings and the jobs drawer from its rows', async () => {
    await render();
    await openDrawer();
    buttonWithText('Settings')?.click();
    await settle();
    expect(nav.goto).toHaveBeenCalledWith('/settings');
    expect(accountDrawer.open).toBe(false);

    jobs.jobs = [job('apply_preset')];
    await openDrawer();
    byLabel('Jobs, 1 job running')?.click();
    await settle();
    expect(jobs.open).toBe(true);
    expect(accountDrawer.open).toBe(false);
  });

  it('marks the Jobs row when a job needs attention', async () => {
    jobs.attention = true;
    await render();
    const element = await openDrawer();

    const row = byLabel('Jobs, a job finished with failures');
    expect(row).not.toBeNull();
    expect(element.querySelector('[data-testid="jobs-attention"]')).not.toBeNull();
  });

  it('signs out at once when no export or upload runs', async () => {
    await render();
    await openDrawer();

    buttonWithText('Sign out')?.click();
    await settle();

    expect(auth.signOut).toHaveBeenCalledOnce();
  });

  it('hands the sign-out to the shared confirm while an export runs', async () => {
    jobs.jobs = [job('export_immich')];
    await render();
    await openDrawer();

    buttonWithText('Sign out')?.click();
    await settle();
    expect(auth.signOut).not.toHaveBeenCalled();
    expect(signOutFlow.confirming).toBe(true);
    expect(accountDrawer.open).toBe(false);
  });

  it('floats a jobs pill while jobs run and opens the jobs drawer from it', async () => {
    await render();
    expect(byLabel('Jobs: Export to Immich · 3 of 12')).toBeNull();

    jobs.jobs = [job('export_immich')];
    await settle();
    const pill = byLabel('Jobs: Export to Immich · 3 of 12');
    expect(pill).not.toBeNull();

    pill?.click();
    await settle();
    expect(jobs.open).toBe(true);
  });

  it('hides the bars while scrolling down in landscape only and makes them inert', async () => {
    layoutState.landscape = true;
    await render();
    const tabBar = target.querySelector<HTMLElement>('nav[aria-label="Main"]');
    const scroller = target.querySelector<HTMLElement>('[data-testid="scroller"]');
    if (!tabBar || !scroller) throw new Error('missing shell');
    const root = document.documentElement.style;
    expect(root.getPropertyValue('--chrome-bottom')).toContain('3.5rem');

    scrollTo(scroller, 10);
    scrollTo(scroller, 200);
    await settle();
    expect(touchBars.hidden).toBe(true);
    expect(tabBar.inert).toBe(true);
    expect(root.getPropertyValue('--chrome-bottom')).toBe('var(--safe-bottom)');

    scrollTo(scroller, 150);
    await settle();
    expect(touchBars.hidden).toBe(false);
    expect(tabBar.inert).toBe(false);
  });

  it('keeps the bars in portrait', async () => {
    await render();
    const scroller = target.querySelector<HTMLElement>('[data-testid="scroller"]');
    if (!scroller) throw new Error('missing shell');

    scrollTo(scroller, 10);
    scrollTo(scroller, 200);
    await settle();
    expect(touchBars.hidden).toBe(false);
  });

  it('clears the chrome offset when it unmounts', async () => {
    await render();
    if (component) void unmount(component);
    component = null;
    await settle();
    expect(document.documentElement.style.getPropertyValue('--chrome-bottom')).toBe('');
  });
});
