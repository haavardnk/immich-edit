import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { touchBars } from '#lib/components/shell/touch/touchBars.svelte.ts';
import { jobs } from '#lib/stores/jobs.svelte.ts';
import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import TouchAppBar from './TouchAppBar.svelte';

type BarProps = ComponentProps<typeof TouchAppBar>;

const Host: Component<{ component: Component<BarProps>; props: BarProps }> = ComponentHost;

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function render(): HTMLElement {
  component = mount(Host, {
    target,
    props: { component: TouchAppBar, props: { title: 'Photos', count: '12 photos' } }
  });
  flushSync();
  const bar = target.querySelector<HTMLElement>('header[data-app-bar]');
  if (!bar) throw new Error('no app bar');
  return bar;
}

beforeEach(() => {
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
  touchBars.set(false);
  jobs.attention = false;
});

describe('TouchAppBar', () => {
  it('shows the title, the count and the avatar', () => {
    const bar = render();

    expect(bar.querySelector('h1')?.textContent).toBe('Photos');
    expect(bar.textContent).toContain('12 photos');
    expect(bar.querySelector('[aria-label="Account"]')).not.toBeNull();
    expect(bar.querySelector('[data-testid="jobs-attention"]')).toBeNull();
  });

  it('marks the avatar when a job needs attention', () => {
    jobs.attention = true;
    const bar = render();

    expect(
      bar.querySelector('[aria-label="Account, a job finished with failures"]')
    ).not.toBeNull();
    expect(bar.querySelector('[data-testid="jobs-attention"]')).not.toBeNull();
  });

  it('is inert while the bars are hidden', () => {
    const bar = render();
    expect(bar.inert).toBe(false);

    touchBars.set(true);
    flushSync();
    expect(bar.inert).toBe(true);
    expect(bar.className).toContain('-mt-13');
  });
});
