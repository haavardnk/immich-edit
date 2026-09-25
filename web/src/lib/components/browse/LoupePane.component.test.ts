import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import LoupePane from './LoupePane.svelte';
import ComponentHost from '$lib/testing/ComponentHost.svelte';
import { CENTERED } from '$lib/stores/compare.svelte';

type PaneProps = ComponentProps<typeof LoupePane>;

const Host: Component<{ component: Component<PaneProps>; props: PaneProps }> = ComponentHost;

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

beforeEach(() => {
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
});

describe('LoupePane', () => {
  it('shows the thumbnail and a busy pane until the preview loads', () => {
    component = mount(Host, {
      target,
      props: {
        component: LoupePane,
        props: { assetId: 'a', alt: 'a.jpg', view: CENTERED, onView: () => {} }
      }
    });
    flushSync();
    const pane = target.querySelector<HTMLElement>('[role="button"]');
    expect(pane?.getAttribute('aria-busy')).toBe('true');
    expect(target.querySelector('[data-testid="loupe-underlay"]')).not.toBeNull();

    target.querySelector('img[alt="a.jpg"]')?.dispatchEvent(new Event('load'));
    flushSync();

    expect(pane?.getAttribute('aria-busy')).toBe('false');
    expect(target.querySelector('[data-testid="loupe-underlay"]')).toBeNull();
  });
});
