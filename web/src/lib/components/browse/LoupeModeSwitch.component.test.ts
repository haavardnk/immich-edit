import { browseView } from '#lib/stores/browseView.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { compare, COMPARE_PANES, MAX_PANES } from '#lib/stores/compare.svelte.ts';
import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { reactiveValue } from '#lib/testing/reactiveValue.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import LoupeModeSwitch from './LoupeModeSwitch.svelte';

type SwitchProps = ComponentProps<typeof LoupeModeSwitch>;

const Host: Component<{ component: Component<SwitchProps>; props: SwitchProps }> = ComponentHost;

function asset(id: string): AssetSummary {
  return { id, originalFileName: `${id}.jpg` } as AssetSummary;
}

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function radios(): HTMLElement[] {
  return [...target.querySelectorAll<HTMLElement>('[role="radio"]')];
}

function radio(name: string): HTMLElement {
  const found = radios().find((candidate) => candidate.textContent.trim() === name);
  if (!found) throw new Error(`no ${name} mode`);
  return found;
}

function press(name: string): void {
  radio(name).click();
  flushSync();
}

function checked(): string[] {
  return radios()
    .filter((candidate) => candidate.getAttribute('aria-checked') === 'true')
    .map((candidate) => candidate.textContent.trim());
}

beforeEach(() => {
  browsing.set(['a', 'b', 'c', 'd', 'e', 'f', 'g'].map(asset));
  browseView.openLoupe('b');
  target = document.createElement('div');
  document.body.appendChild(target);
  component = mount(Host, { target, props: { component: LoupeModeSwitch, props: {} } });
  flushSync();
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
  browseView.closeLoupe();
});

describe('LoupeModeSwitch', () => {
  it('is one radio group with a short label per view mode and checks the current one', () => {
    expect(target.querySelector('[role="radiogroup"]')?.getAttribute('aria-label')).toBe(
      'View mode'
    );
    expect(radios().map((candidate) => candidate.textContent.trim())).toEqual([
      'Single',
      'Compare',
      'Survey'
    ]);
    expect(checked()).toEqual(['Single']);
  });

  it('roves and switches mode with the arrow keys', () => {
    const single = radio('Single');
    single.focus();
    single.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true }));
    flushSync();
    expect(document.activeElement).toBe(radio('Compare'));
    expect(compare.mode).toBe('compare');
    expect(checked()).toEqual(['Compare']);
  });

  it('enters compare and survey from the photo on screen and returns to single', () => {
    press('Compare');
    expect(compare.mode).toBe('compare');
    expect(compare.members).toEqual(['b', 'c']);
    expect(checked()).toEqual(['Compare']);

    press('Survey');
    expect(compare.mode).toBe('survey');
    expect(checked()).toEqual(['Survey']);

    press('Single');
    expect(compare.mode).toBe('single');
    expect(browseView.loupeId).toBe('b');
  });

  it('caps the survey at the layout limit', () => {
    const release = compare.limitBy(() => 4);
    try {
      press('Survey');
      expect(compare.members).toEqual(['b', 'c', 'd', 'e']);
    } finally {
      release();
    }
  });

  it('offers only Single and Compare when the layout holds two panes', () => {
    const release = compare.limitBy(() => COMPARE_PANES);
    try {
      flushSync();
      expect(radios().map((candidate) => candidate.textContent.trim())).toEqual([
        'Single',
        'Compare'
      ]);
      press('Compare');
      expect(compare.members).toEqual(['b', 'c']);
    } finally {
      release();
    }
  });

  it('keeps a running survey checked when the layout narrows to two panes', () => {
    const limit = reactiveValue(MAX_PANES);
    const release = compare.limitBy(() => limit.value);
    try {
      press('Survey');
      limit.value = COMPARE_PANES;
      flushSync();
      expect(checked()).toEqual(['Survey']);
      expect(compare.members).toEqual(['b', 'c', 'd', 'e', 'f', 'g']);
      press('Compare');
      expect(radios().map((candidate) => candidate.textContent.trim())).toEqual([
        'Single',
        'Compare'
      ]);
    } finally {
      release();
    }
  });
});
