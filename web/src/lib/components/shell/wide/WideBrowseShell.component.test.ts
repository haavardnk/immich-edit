import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { createRawSnippet, flushSync, mount, unmount, type Component, type Snippet } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import WideBrowseShell from './WideBrowseShell.svelte';
import { accountDrawer } from '#lib/components/shell/account/accountDrawer.svelte.ts';
import { JOB_KINDS, type Job } from '#lib/api/jobs.ts';
import { jobs } from '#lib/stores/jobs.svelte.ts';
import { session } from '#lib/stores/session.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import { resetLayout } from '#lib/testing/layoutMock.ts';

const route = vi.hoisted(() => ({ page: { url: new URL('http://localhost/photos'), params: {} } }));

vi.mock('$app/state', () => route);

vi.mock('#lib/device/layout.svelte.ts', async () => ({
  layout: (await import('#lib/testing/layoutMock.ts')).fakeLayout
}));

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
    props: { component: WideBrowseShell, props: { children } }
  });
  flushSync();
}

function labelled(name: string): HTMLElement[] {
  return [...target.querySelectorAll<HTMLElement>(`[aria-label="${name}"]`)];
}

const runningJob: Job = {
  id: 'j1',
  kind: JOB_KINDS.exportImmich,
  status: 'running',
  target: null,
  params: null,
  total: 12,
  completed: 3,
  failed: 0,
  cancelled_at: null,
  created_at: '2026-11-05T00:00:00Z',
  updated_at: '2026-11-05T00:00:00Z'
};

beforeEach(() => {
  vi.stubGlobal(
    'fetch',
    vi.fn(() => Promise.reject(new Error('offline')))
  );
  resetLayout({ mode: 'touch', wide: true, landscape: true });
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
  vi.unstubAllGlobals();
  resetLayout();
  jobs.jobs = [];
  ui.searchQuery = '';
  accountDrawer.close();
  flushSync();
});

describe('WideBrowseShell', () => {
  it('puts the brand, search and library in the sidebar and no top bar', () => {
    render('/photos');

    expect(target.querySelector('[aria-label="Global navigation"]')).toBeNull();
    expect(target.querySelector('main [data-testid="route"]')).not.toBeNull();
    const library = target.querySelector('aside[aria-label="Library"]');
    const sidebar = library?.parentElement?.parentElement;
    expect(sidebar?.querySelector('[aria-label="immich-edit"]')).not.toBeNull();
    expect(sidebar?.querySelector('[role="search"] input')).not.toBeNull();
    expect(target.querySelector('main [role="search"]')).toBeNull();
  });

  it('shows the administration sidebar on settings routes', () => {
    render('/settings');

    expect(target.querySelector('aside[aria-label="Administration"]')).not.toBeNull();
    expect(target.querySelector('aside[aria-label="Library"]')).toBeNull();
  });

  it('fills the search field from the search route', () => {
    render('/search?q=fjord');

    const input = target.querySelector<HTMLInputElement>('[role="search"] input');
    expect(input?.value).toBe('fjord');
    expect(labelled('Clear search')).toHaveLength(1);
  });

  it('has the account, settings and running jobs once each at the sidebar foot', () => {
    session.set({ id: 'u', email: 'h@x', name: 'Hanne', is_admin: false, auth_kind: 'password' });
    jobs.jobs = [runningJob];
    render('/photos');

    expect(labelled('Settings')).toHaveLength(1);
    expect(labelled('Jobs')).toHaveLength(0);
    expect(labelled('Jobs: Exporting 3 of 12')).toHaveLength(1);
    expect(target.textContent).toContain('Hanne');
    expect(target.textContent).toContain('localhost');

    const [account] = labelled('Account');
    expect(labelled('Account')).toHaveLength(1);
    account?.click();
    flushSync();
    expect(accountDrawer.open).toBe(true);
    expect(document.querySelector('[role="dialog"]')?.textContent).toContain('Sign out');
  });

  it('opens the jobs list from the jobs card', () => {
    jobs.jobs = [runningJob];
    render('/photos');

    labelled('Jobs: Exporting 3 of 12')[0]?.click();
    flushSync();
    expect(jobs.open).toBe(true);
    jobs.close();
  });

  it('hides the jobs card when nothing runs', () => {
    render('/photos');

    expect(target.querySelector('[aria-label^="Jobs:"]')).toBeNull();
  });
});
