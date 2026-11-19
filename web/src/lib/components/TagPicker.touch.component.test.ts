import TagPicker from '#lib/components/TagPicker.svelte';
import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { layoutState, resetLayout } from '#lib/testing/layoutMock.ts';
import type { TagRef } from '#lib/types/asset.ts';
import { flushSync, mount, tick, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('#lib/device/layout.svelte.ts', async () => ({
  layout: (await import('#lib/testing/layoutMock.ts')).fakeLayout
}));

vi.mock('#lib/stores/library.svelte.ts', () => ({
  library: {
    tags: [
      { id: 'tag-sky', name: 'Sky', value: 'Sky', createdAt: null },
      { id: 'tag-places', name: 'Places', value: 'Places', createdAt: null },
      { id: 'tag-sea', name: 'Sea', value: 'Places/Sea', createdAt: null }
    ],
    ensureTags: () => Promise.resolve()
  }
}));

vi.mock('#lib/stores/recentTags.svelte.ts', () => ({
  recentTags: {
    tags: [{ id: 'tag-sky', name: 'Sky', value: 'Sky' }],
    record: () => {}
  }
}));

type PickerProps = ComponentProps<typeof TagPicker>;

const Host: Component<{ component: Component<PickerProps>; props: PickerProps }> = ComponentHost;

const SEA: TagRef = { id: 'tag-sea', name: 'Sea', value: 'Places/Sea' };

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

async function settle(): Promise<void> {
  flushSync();
  await tick();
  await new Promise((resolve) => setTimeout(resolve, 0));
  flushSync();
}

async function render(open: boolean, tags: TagRef[] = []) {
  const onAdd = vi.fn<(tag: TagRef) => void>();
  const props: PickerProps = {
    tags,
    open,
    onToggle: () => {},
    onClose: () => {},
    onAdd,
    onRemove: () => {},
    onCreate: () => null,
    subtitle: 'IMG_4120.ARW'
  };
  component = mount(Host, { target, props: { component: TagPicker, props } });
  await settle();
  return { onAdd };
}

function dialog(): HTMLElement {
  const element = document.querySelector<HTMLElement>('[role="dialog"]');
  if (!element) throw new Error('no dialog');
  return element;
}

function headings(root: ParentNode): string[] {
  return [...root.querySelectorAll('h3')].map((heading) => heading.textContent.trim());
}

beforeEach(() => {
  resetLayout({ mode: 'touch' });
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
  document.body.innerHTML = '';
});

describe('TagPicker on touch', () => {
  it('fills one sheet with the search, applied, recent and the tag tree', async () => {
    await render(true, [SEA]);
    const sheet = dialog();
    const titleId = sheet.getAttribute('aria-labelledby');
    expect(titleId && document.getElementById(titleId)?.textContent.trim()).toBe('Tags');
    expect(sheet.textContent).toContain('IMG_4120.ARW');
    expect(sheet.querySelector('input[aria-label="Add tags…"]')).not.toBeNull();
    expect(headings(sheet)).toEqual(['Applied', 'Recent', 'All tags']);
    expect(sheet.querySelector('[aria-label="Remove Places/Sea"]')).not.toBeNull();
    const recent = sheet.querySelector<HTMLElement>('button[aria-pressed][aria-label="Sky"]');
    expect(recent?.textContent.trim()).toBe('Sky');
    expect(document.querySelectorAll('[role="dialog"]')).toHaveLength(1);
  });

  it('adds a tag with one tap on its tree row', async () => {
    const { onAdd } = await render(true);
    const label = [...dialog().querySelectorAll<HTMLLabelElement>('label')].find(
      (item) => item.textContent.trim() === 'Sky'
    );
    const box = label && document.getElementById(label.htmlFor);
    box?.click();
    await settle();
    expect(onAdd).toHaveBeenCalledWith(expect.objectContaining({ id: 'tag-sky' }));
  });

  it('keeps the anchored panel without the applied section on desktop', async () => {
    layoutState.mode = 'desktop';
    await render(true, [SEA]);
    expect(document.querySelector('button[data-sheet-handle]')).toBeNull();
    const panel = document.querySelector<HTMLElement>('[data-popover-content]');
    expect(panel?.querySelector('input[aria-label="Add tags…"]')).not.toBeNull();
    expect(headings(panel ?? document)).toEqual(['Recent', 'All tags']);
  });
});
