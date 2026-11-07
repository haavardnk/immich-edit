import { compare } from '#lib/stores/compare.svelte.ts';
import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import LoupePaneRail from './LoupePaneRail.svelte';

type RailProps = ComponentProps<typeof LoupePaneRail>;

const Host: Component<{ component: Component<RailProps>; props: RailProps }> = ComponentHost;

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function render(props: RailProps): void {
  component = mount(Host, { target, props: { component: LoupePaneRail, props } });
  flushSync();
}

function button(name: string): HTMLButtonElement {
  const found = [...target.querySelectorAll('button')].find(
    (candidate) => candidate.textContent.trim() === name
  );
  if (!found) throw new Error(`no ${name}`);
  return found;
}

beforeEach(() => {
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
  compare.exit();
});

describe('LoupePaneRail', () => {
  it('names the focused compare pane by number and file stem with labelled actions', () => {
    compare.enter('compare', ['a', 'b'], 1);
    render({ fileName: 'IMG_4121.ARW' });
    expect(target.querySelector('[role="img"]')?.getAttribute('aria-label')).toBe('Pane 2');
    expect(target.textContent).toContain('IMG_4121');
    expect(target.textContent).not.toContain('.ARW');
    expect(button('Sync').getAttribute('aria-pressed')).toBe('true');
    expect(button('Promote').disabled).toBe(false);
    button('Promote').click();
    flushSync();
    expect(compare.members).toEqual(['b', 'a']);
    expect(button('Promote').disabled).toBe(true);
  });

  it('names the focused survey pane by file stem and offers Keep and Drop', () => {
    compare.enter('survey', ['a', 'b', 'c', 'd'], 2);
    render({ fileName: 'IMG_0003.ARW' });
    expect(target.querySelector('[role="img"]')?.getAttribute('aria-label')).toBe('Pane 3');
    expect(target.textContent).toContain('IMG_0003');
    expect(target.textContent).not.toContain('of 4');
    button('Drop').click();
    flushSync();
    expect(compare.members).toEqual(['a', 'b', 'd']);
    button('Keep').click();
    flushSync();
    expect(compare.members).toEqual(['d']);
  });

  it('toggles synced zoom', () => {
    compare.enter('compare', ['a', 'b']);
    render({ fileName: 'a.jpg' });
    button('Sync').click();
    flushSync();
    expect(compare.syncView).toBe(false);
    expect(button('Sync').getAttribute('aria-pressed')).toBe('false');
    compare.syncView = true;
  });
});
