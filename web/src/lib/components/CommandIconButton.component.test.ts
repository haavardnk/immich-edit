import type { IconCommand } from '#lib/commands/types.ts';
import { hint } from '#lib/shortcuts/labels.ts';
import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { reactiveFlag } from '#lib/testing/reactiveFlag.svelte.ts';
import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import CommandIconButton from './CommandIconButton.svelte';

type ButtonProps = ComponentProps<typeof CommandIconButton>;

const Host: Component<{ component: Component<ButtonProps>; props: ButtonProps }> = ComponentHost;

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function command(spec: Partial<IconCommand> = {}): IconCommand {
  return { id: 'test', contexts: ['global'], label: 'Undo', icon: 'M0 0', run: vi.fn(), ...spec };
}

function render(props: ButtonProps): HTMLButtonElement {
  component = mount(Host, { target, props: { component: CommandIconButton, props } });
  flushSync();
  const button = target.querySelector('button');
  if (!button) throw new Error('no button');
  return button;
}

function pointer(element: HTMLElement, type: string): void {
  element.dispatchEvent(new Event(type, { bubbles: true }));
  flushSync();
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

describe('CommandIconButton', () => {
  it('labels itself with the command title and runs on click', () => {
    const undo = command({ bind: 'undo' });
    const button = render({ command: undo, size: 'small', class: 'shrink-0' });
    expect(button.getAttribute('aria-label')).toBe(hint('Undo', 'undo'));
    expect(button.classList.contains('shrink-0')).toBe(true);
    expect(button.hasAttribute('aria-pressed')).toBe(false);
    button.click();
    expect(undo.run).toHaveBeenCalledOnce();
  });

  it('follows enabled and shows the unavailable reason', () => {
    const flag = reactiveFlag(false);
    const gated = command({ enabled: () => flag.value, unavailable: () => 'Nothing to undo' });
    const button = render({ command: gated });
    expect(button.disabled).toBe(true);
    expect(button.getAttribute('aria-label')).toBe('Nothing to undo');
    button.click();
    expect(gated.run).not.toHaveBeenCalled();
    flag.value = true;
    flushSync();
    expect(button.disabled).toBe(false);
    expect(button.getAttribute('aria-label')).toBe('Undo');
    button.click();
    expect(gated.run).toHaveBeenCalledOnce();
  });

  it('turns primary and pressed while the command is pressed', () => {
    const flag = reactiveFlag(false);
    const button = render({ command: command({ pressed: () => flag.value }) });
    const idle = button.className;
    expect(button.getAttribute('aria-pressed')).toBe('false');
    flag.value = true;
    flushSync();
    expect(button.getAttribute('aria-pressed')).toBe('true');
    expect(button.className).not.toBe(idle);
    expect(button.className).toMatch(/primary/);
    expect(idle).not.toMatch(/primary/);
  });

  it('runs a hold command on press and releases it on lift, leave or cancel', () => {
    const release = vi.fn();
    const hold = command({ release });
    const button = render({ command: hold });
    pointer(button, 'pointerdown');
    expect(hold.run).toHaveBeenCalledOnce();
    expect(release).not.toHaveBeenCalled();
    pointer(button, 'pointerup');
    pointer(button, 'pointerleave');
    pointer(button, 'pointercancel');
    expect(release).toHaveBeenCalledTimes(3);
    button.click();
    expect(hold.run).toHaveBeenCalledOnce();
  });

  it('runs the before hook ahead of the command', () => {
    const order: string[] = [];
    const button = render({
      command: command({ run: () => order.push('run') }),
      before: () => order.push('before')
    });
    button.click();
    expect(order).toEqual(['before', 'run']);
  });
});
