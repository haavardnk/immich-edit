import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import BackButton from './BackButton.svelte';

const nav = vi.hoisted(() => ({ goto: vi.fn() }));

vi.mock('$app/state', () => ({ page: { url: new URL('http://localhost/people/p1'), params: {} } }));

vi.mock('$app/navigation', () => ({ goto: nav.goto }));

type BackProps = ComponentProps<typeof BackButton>;

const Host: Component<{ component: Component<BackProps>; props: BackProps }> = ComponentHost;

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function render(props: BackProps): HTMLButtonElement {
  component = mount(Host, { target, props: { component: BackButton, props } });
  flushSync();
  const button = target.querySelector('button');
  if (!button) throw new Error('no back button');
  return button;
}

beforeEach(() => {
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
  nav.goto.mockClear();
});

describe('BackButton', () => {
  it('goes to the logical parent by default', () => {
    render({}).click();

    expect(nav.goto).toHaveBeenCalledWith('/people', { replace: true });
  });

  it('runs the override instead of the shared back', () => {
    const onBack = vi.fn();
    const button = render({ labelled: true, onBack });

    expect(button.textContent?.trim()).toBe('Back');
    button.click();

    expect(onBack).toHaveBeenCalledOnce();
    expect(nav.goto).not.toHaveBeenCalled();
  });
});
