import type { IconCommand } from '#lib/commands/types.ts';
import { keysFor } from '#lib/shortcuts/labels.ts';
import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { reactiveFlag } from '#lib/testing/reactiveFlag.svelte.ts';
import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import CommandList from './CommandList.svelte';

type ListProps = ComponentProps<typeof CommandList>;

const Host: Component<{ component: Component<ListProps>; props: ListProps }> = ComponentHost;

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function command(id: string, spec: Partial<IconCommand> = {}): IconCommand {
  return { id, contexts: ['global'], label: id, icon: 'M0 0', run: vi.fn(), ...spec };
}

function render(props: ListProps): void {
  component = mount(Host, { target, props: { component: CommandList, props } });
  flushSync();
}

function row(label: string): HTMLButtonElement {
  const button = target.querySelector<HTMLButtonElement>(`button[aria-label="${label}"]`);
  if (!button) throw new Error(`no row ${label}`);
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

describe('CommandList', () => {
  it('renders groups in order with a divider between them', () => {
    render({ menu: [[command('A'), command('B')], [command('C')], [command('D')]] });
    const sequence = [...target.querySelectorAll('button, div.border-t')].map((element) =>
      element.tagName === 'BUTTON' ? element.getAttribute('aria-label') : '|'
    );
    expect(sequence).toEqual(['A', 'B', '|', 'C', '|', 'D']);
  });

  it('labels rows from the command and shows bound keys', () => {
    render({
      menu: [[command('undo', { label: () => 'Dynamic', bind: 'undo' }), command('plain')]]
    });
    expect(row('Dynamic').textContent).toContain(keysFor('undo'));
    expect(row('plain').textContent.trim()).toBe('plain');
  });

  it('reflects pressed and enabled state', () => {
    render({
      menu: [
        [
          command('on', { pressed: () => true }),
          command('off', { pressed: () => false, enabled: () => false }),
          command('none')
        ]
      ]
    });
    expect(row('on').getAttribute('aria-pressed')).toBe('true');
    expect(row('off').getAttribute('aria-pressed')).toBe('false');
    expect(row('none').hasAttribute('aria-pressed')).toBe(false);
    expect(row('on').disabled).toBe(false);
    expect(row('off').disabled).toBe(true);
  });

  it('updates a row when its command state changes', () => {
    const flag = reactiveFlag(false);
    render({
      menu: [
        [
          command('live', {
            label: () => (flag.value ? 'Live on' : 'Live off'),
            pressed: () => flag.value,
            enabled: () => flag.value
          })
        ]
      ]
    });
    expect(row('Live off').getAttribute('aria-pressed')).toBe('false');
    expect(row('Live off').disabled).toBe(true);
    flag.value = true;
    flushSync();
    expect(target.querySelector('button[aria-label="Live off"]')).toBeNull();
    expect(row('Live on').getAttribute('aria-pressed')).toBe('true');
    expect(row('Live on').disabled).toBe(false);
  });

  it('calls onrun and then runs the command on click', () => {
    const calls: string[] = [];
    const run = command('run', { run: () => calls.push('run') });
    render({ menu: [[run]], onrun: () => calls.push('onrun') });
    row('run').click();
    expect(calls).toEqual(['onrun', 'run']);
  });

  it('does not run a disabled command', () => {
    const blocked = command('blocked', { enabled: () => false });
    const onrun = vi.fn();
    render({ menu: [[blocked]], onrun });
    row('blocked').click();
    expect(blocked.run).not.toHaveBeenCalled();
    expect(onrun).not.toHaveBeenCalled();
  });

  it('runs a hold row on press and releases it on lift or leave', () => {
    const release = vi.fn();
    const hold = command('hold', { release });
    const onrun = vi.fn();
    render({ menu: [[hold]], onrun });
    pointer(row('hold'), 'pointerdown');
    expect(hold.run).toHaveBeenCalledOnce();
    expect(release).not.toHaveBeenCalled();
    pointer(row('hold'), 'pointerup');
    expect(release).toHaveBeenCalledOnce();
    pointer(row('hold'), 'pointerleave');
    pointer(row('hold'), 'pointercancel');
    expect(release).toHaveBeenCalledTimes(3);
    row('hold').click();
    expect(hold.run).toHaveBeenCalledOnce();
    expect(onrun).not.toHaveBeenCalled();
  });

  it('wraps only the rows that rowClass names', () => {
    const wide = command('wide');
    render({
      menu: [[command('narrow'), wide]],
      rowClass: (entry) => (entry === wide ? 'xl:hidden' : undefined)
    });
    expect(row('wide').parentElement?.className).toBe('contents xl:hidden');
    expect(row('narrow').parentElement?.classList.contains('contents')).toBe(false);
    expect(target.querySelectorAll('div.contents')).toHaveLength(1);
  });
});
