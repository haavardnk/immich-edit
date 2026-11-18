import ComponentHost from '#lib/testing/ComponentHost.svelte';
import {
  createRawSnippet,
  flushSync,
  mount,
  unmount,
  type Component,
  type ComponentProps
} from 'svelte';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import TooltipLink from './TooltipLink.svelte';

type LinkProps = ComponentProps<typeof TooltipLink>;

const Host: Component<{ component: Component<LinkProps>; props: LinkProps }> = ComponentHost;

const label = createRawSnippet(() => ({ render: () => '<span>Bergen</span>' }));

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function render(props: Omit<LinkProps, 'children'>): HTMLAnchorElement {
  component = mount(Host, {
    target,
    props: { component: TooltipLink, props: { ...props, children: label } }
  });
  flushSync();
  const link = target.querySelector('a');
  if (!link) throw new Error('no link');
  return link;
}

beforeEach(() => {
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
});

describe('TooltipLink', () => {
  it('renders a plain link without the trigger button type', () => {
    for (const always of [false, true]) {
      const link = render({ tooltip: 'Bergen', href: '/albums/bergen', always });
      expect(link.getAttribute('href')).toBe('/albums/bergen');
      expect(link.hasAttribute('type')).toBe(false);
      if (component) void unmount(component);
      component = null;
    }
  });

  it('keeps an explicit media type', () => {
    const link = render({ tooltip: 'Bergen', href: '/a.jpg', type: 'image/jpeg' });
    expect(link.getAttribute('type')).toBe('image/jpeg');
  });
});
