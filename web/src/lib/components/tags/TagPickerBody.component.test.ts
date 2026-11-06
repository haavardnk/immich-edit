import type { TagSummary } from '#lib/api/tags.ts';
import TagPickerBody from '#lib/components/tags/TagPickerBody.svelte';
import ComponentHost from '#lib/testing/ComponentHost.svelte';
import type { TagRef } from '#lib/types/asset.ts';
import { flushSync, mount, tick, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const recent = vi.hoisted(() => ({
  tags: [] as TagRef[],
  record: vi.fn((_tag: TagRef) => {})
}));

vi.mock('#lib/stores/recentTags.svelte.ts', () => ({ recentTags: recent }));

type BodyProps = ComponentProps<typeof TagPickerBody>;

const Host: Component<{ component: Component<BodyProps>; props: BodyProps }> = ComponentHost;

function summary(id: string, value: string): TagSummary {
  return { id, name: value.split('/').pop() ?? value, value, createdAt: null };
}

const ALL = [
  summary('places', 'Places'),
  summary('norway', 'Places/Norway'),
  summary('lofoten', 'Places/Norway/Lofoten'),
  summary('iceland', 'Places/Iceland'),
  summary('fjord', 'Fjord')
];

const LOFOTEN: TagRef = { id: 'lofoten', name: 'Lofoten', value: 'Places/Norway/Lofoten' };
const FJORD: TagRef = { id: 'fjord', name: 'Fjord', value: 'Fjord' };

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

async function settle(): Promise<void> {
  flushSync();
  await tick();
  await new Promise((resolve) => setTimeout(resolve, 0));
  flushSync();
}

async function render(overrides: Partial<BodyProps> = {}) {
  const onAdd = vi.fn<(tag: TagRef) => void>();
  const onRemove = vi.fn<(tagId: string) => void>();
  const onCreate = vi.fn<(value: string) => TagRef | null>(() => ({
    id: 'new',
    name: 'Dusk',
    value: 'Dusk'
  }));
  const props: BodyProps = {
    all: ALL,
    applied: [LOFOTEN],
    showApplied: true,
    numbered: true,
    onAdd,
    onRemove,
    onCreate,
    ...overrides
  };
  component = mount(Host, { target, props: { component: TagPickerBody, props } });
  await settle();
  return { onAdd, onRemove, onCreate };
}

function input(): HTMLInputElement {
  const element = target.querySelector<HTMLInputElement>('input[aria-label="Add tags…"]');
  if (!element) throw new Error('no search');
  return element;
}

async function type(value: string): Promise<void> {
  input().value = value;
  input().dispatchEvent(new Event('input', { bubbles: true }));
  await settle();
}

async function press(key: string, init: KeyboardEventInit = {}): Promise<KeyboardEvent> {
  const event = new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true, ...init });
  input().dispatchEvent(event);
  await settle();
  return event;
}

function box(name: string): HTMLElement | null {
  const label = [...target.querySelectorAll<HTMLLabelElement>('label')].find(
    (item) => item.textContent.trim().split(/\s+/)[0] === name
  );
  return label ? document.getElementById(label.htmlFor) : null;
}

beforeEach(() => {
  recent.tags = [FJORD, LOFOTEN];
  recent.record.mockClear();
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
});

describe('TagPickerBody', () => {
  it('opens the tree on the path of an applied tag with mixed parents', async () => {
    await render();
    expect(box('Places')?.getAttribute('aria-checked')).toBe('mixed');
    expect(box('Norway')?.getAttribute('aria-checked')).toBe('mixed');
    expect(box('Lofoten')?.getAttribute('aria-checked')).toBe('true');
    expect(box('Iceland')?.getAttribute('aria-checked')).toBe('false');
    expect(box('Fjord')?.getAttribute('aria-checked')).toBe('false');
  });

  it('collapses a branch and keeps the rows still while a tag is toggled', async () => {
    const { onRemove } = await render();
    target.querySelector<HTMLElement>('[aria-label="Collapse Norway"]')?.click();
    await settle();
    expect(box('Lofoten')).toBeNull();
    target.querySelector<HTMLElement>('[aria-label="Expand Norway"]')?.click();
    await settle();
    box('Lofoten')?.click();
    await settle();
    expect(onRemove).toHaveBeenCalledWith('lofoten');
    expect(box('Lofoten')).not.toBeNull();
  });

  it('numbers the recent chips, marks the applied one and toggles it', async () => {
    const { onAdd, onRemove } = await render();
    const chips = [...target.querySelectorAll<HTMLElement>('button[aria-pressed]')];
    expect(chips.map((chip) => chip.textContent.replace(/\s+/g, ' ').trim())).toEqual([
      '1 Fjord',
      '2 Lofoten'
    ]);
    expect(chips.map((chip) => chip.getAttribute('aria-pressed'))).toEqual(['false', 'true']);
    chips[0]?.click();
    chips[1]?.click();
    expect(onAdd).toHaveBeenCalledWith(FJORD);
    expect(onRemove).toHaveBeenCalledWith('lofoten');
    expect(recent.record).not.toHaveBeenCalled();
  });

  it('adds a recent tag with Alt and its digit, by key code', async () => {
    const { onAdd } = await render();
    const event = await press('¡', { code: 'Digit1', altKey: true });
    expect(onAdd).toHaveBeenCalledWith(FJORD);
    expect(event.defaultPrevented).toBe(true);
    await press('™', { code: 'Digit3', altKey: true });
    expect(onAdd).toHaveBeenCalledTimes(1);
  });

  it('clears every applied tag', async () => {
    const { onRemove } = await render({ applied: [LOFOTEN, FJORD] });
    [...target.querySelectorAll<HTMLElement>('button')]
      .find((button) => button.textContent.trim() === 'Clear')
      ?.click();
    expect(onRemove.mock.calls.map(([id]) => id)).toEqual(['lofoten', 'fjord']);
  });

  it('shows plus chips and no applied section when the container asks', async () => {
    await render({ showApplied: false, numbered: false });
    const titles = [...target.querySelectorAll('h3')].map((h) => h.textContent.trim());
    expect(titles).toEqual(['Recent', 'All tags']);
    expect(target.querySelector('button[aria-pressed]')?.textContent.trim()).toBe('Fjord');
  });

  it('searches flat matches and adds the first one on Enter as a recent tag', async () => {
    const { onAdd } = await render();
    await type('lan');
    const titles = [...target.querySelectorAll('h3')].map((h) => h.textContent.trim());
    expect(titles).toEqual(['Applied', 'Matches']);
    expect(target.querySelector('mark')?.textContent).toBe('lan');
    await press('Enter');
    expect(onAdd).toHaveBeenCalledWith(expect.objectContaining({ id: 'iceland' }));
    expect(recent.record).toHaveBeenCalledWith(expect.objectContaining({ id: 'iceland' }));
    expect(input().value).toBe('');
  });

  it('creates a missing tag on Enter and records it', async () => {
    const { onCreate } = await render();
    await type('Dusk');
    expect(target.textContent).toContain('Create “Dusk”');
    await press('Enter');
    expect(onCreate).toHaveBeenCalledWith('Dusk');
    expect(recent.record).toHaveBeenCalledWith(expect.objectContaining({ id: 'new' }));
  });
});
