import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import DateScrubber from './DateScrubber.svelte';
import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { resetLayout } from '#lib/testing/layoutMock.ts';
import { monthCounts } from '#lib/stores/monthCounts.svelte.ts';
import type { BrowseFeed } from '#lib/stores/browseFeed.svelte.ts';

vi.mock('#lib/device/layout.svelte.ts', async () => ({
  layout: (await import('#lib/testing/layoutMock.ts')).fakeLayout
}));

const api = vi.hoisted(() => ({
  searchMonths: vi.fn(async () => ({
    months: [
      { month: '2026-10', count: 3 },
      { month: '2025-07', count: 1 }
    ]
  })),
  searchFirst: vi.fn(async (): Promise<string | null> => null)
}));

vi.mock('#lib/api/search.ts', () => api);

type Props = ComponentProps<typeof DateScrubber>;

const Host: Component<{ component: Component<Props>; props: Props }> = ComponentHost;

let run = 0;

function timelineFeed(): BrowseFeed {
  const filename = `run-${(run += 1)}`;
  return {
    hasPrevious: false,
    metadataBacked: true,
    currentFilter: () => ({ filename }),
    jumpTo: vi.fn(async () => true)
  } as unknown as BrowseFeed;
}

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function render(): void {
  component = mount(Host, {
    target,
    props: { component: DateScrubber, props: { feed: timelineFeed(), scroller: undefined } }
  });
  flushSync();
}

beforeEach(() => {
  monthCounts.months = [];
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
  resetLayout();
});

describe('DateScrubber', () => {
  it('shows the persistent rail on desktop with the month as its value', async () => {
    render();
    await vi.waitFor(() =>
      expect(target.querySelector('[data-testid="date-rail"]')).not.toBeNull()
    );
    const rail = target.querySelector('[role="slider"]');
    expect(rail?.getAttribute('aria-label')).toBe('Timeline');
    expect(rail?.getAttribute('aria-valuetext')).toBe('October 2026');
    expect(rail?.getAttribute('aria-valuemax')).toBe('1');
    expect(target.querySelector('[data-testid="date-thumb"]')).toBeNull();
  });

  it('shows the scroll thumb on touch', async () => {
    resetLayout({ mode: 'touch' });
    render();
    await vi.waitFor(() =>
      expect(target.querySelector('[data-testid="date-thumb"]')).not.toBeNull()
    );
    expect(target.querySelector('[data-testid="date-rail"]')).toBeNull();
  });

  it('steps a month from the keyboard and searches for the unloaded one on key up', async () => {
    render();
    await vi.waitFor(() => expect(target.querySelector('[role="slider"]')).not.toBeNull());
    const rail = target.querySelector<HTMLElement>('[role="slider"]');
    rail?.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }));
    flushSync();
    expect(rail?.getAttribute('aria-valuetext')).toBe('July 2025');
    expect(api.searchFirst).not.toHaveBeenCalled();
    rail?.dispatchEvent(new KeyboardEvent('keyup', { key: 'ArrowDown', bubbles: true }));
    expect(api.searchFirst).toHaveBeenCalledWith(
      expect.objectContaining({ takenBefore: '2025-07-31T23:59:59.999Z' }),
      expect.objectContaining({ field: 'taken' })
    );
  });
});
