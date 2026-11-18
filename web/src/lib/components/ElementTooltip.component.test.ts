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
import ElementTooltip from './ElementTooltip.svelte';
import TruncationTooltip from './TruncationTooltip.svelte';

type TooltipProps = ComponentProps<typeof ElementTooltip>;

type ChildSnippet = TooltipProps['child'];

type TriggerProps = { text: string; force?: boolean; child: ChildSnippet };

const Host: Component<{ component: Component<TriggerProps>; props: TriggerProps }> = ComponentHost;

const heading = createRawSnippet((props: () => { props: Record<string, unknown> }) => ({
  render: () => '<h2>Bergen</h2>',
  setup: (node) => {
    for (const [key, value] of Object.entries(props().props)) {
      if (typeof value === 'string') node.setAttribute(key, value);
    }
  }
}));

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function render(tooltip: Component<TriggerProps>, props: Omit<TriggerProps, 'child'>): Element {
  component = mount(Host, {
    target,
    props: { component: tooltip, props: { ...props, child: heading } }
  });
  flushSync();
  const element = target.querySelector('h2');
  if (!element) throw new Error('no heading');
  return element;
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

describe('ElementTooltip', () => {
  it('hands the trigger props to any element without the button type', () => {
    const element = render(ElementTooltip, { text: 'Bergen' });
    expect(element.hasAttribute('data-tooltip-trigger')).toBe(true);
    expect(element.hasAttribute('type')).toBe(false);
  });

  it('keeps truncation tooltips off the button type too', () => {
    const element = render(TruncationTooltip, { text: 'Bergen', force: true });
    expect(element.hasAttribute('data-tooltip-trigger')).toBe(true);
    expect(element.hasAttribute('type')).toBe(false);
  });
});
