import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import IconLink from './IconLink.svelte';

type LinkProps = ComponentProps<typeof IconLink>;

const Host: Component<{ component: typeof IconLink; props: LinkProps }> = ComponentHost;

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

describe('IconLink', () => {
  it('renders a tooltip link without the button type', () => {
    component = mount(Host, {
      target,
      props: {
        component: IconLink,
        props: {
          href: '/settings',
          icon: 'M0 0',
          'aria-label': 'Settings',
          tooltip: 'Open'
        }
      }
    });
    flushSync();
    const link = target.querySelector('a');
    expect(link?.getAttribute('href')).toBe('/settings');
    expect(link?.getAttribute('aria-label')).toBe('Settings');
    expect(link?.hasAttribute('data-tooltip-trigger')).toBe(true);
    expect(link?.hasAttribute('type')).toBe(false);
    expect(link?.hasAttribute('title')).toBe(false);
  });
});
