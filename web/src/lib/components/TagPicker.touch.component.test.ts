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
      { id: 'tag-sea', name: 'Sea', value: 'Places/Sea', createdAt: null }
    ],
    ensureTags: () => Promise.resolve()
  }
}));

type PickerProps = ComponentProps<typeof TagPicker>;

const Host: Component<{ component: Component<PickerProps>; props: PickerProps }> = ComponentHost;

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

async function settle(): Promise<void> {
  flushSync();
  await tick();
  await new Promise((resolve) => setTimeout(resolve, 0));
  flushSync();
}

async function render(open: boolean) {
  const onAdd = vi.fn<(tag: TagRef) => void>();
  const props: PickerProps = {
    tags: [],
    open,
    onToggle: () => {},
    onClose: () => {},
    onAdd,
    onRemove: () => {},
    onCreate: () => null
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
  it('shows the search and the tag list inline in its sheet', async () => {
    await render(true);
    const sheet = dialog();
    const titleId = sheet.getAttribute('aria-labelledby');
    expect(titleId && document.getElementById(titleId)?.textContent.trim()).toBe('Tags');
    expect(sheet.querySelector('input[role="combobox"]')).not.toBeNull();
    expect(sheet.querySelector('[role="listbox"]')).not.toBeNull();
    const options = [...sheet.querySelectorAll('[role="option"]')].map((option) =>
      option.textContent.trim()
    );
    expect(options).toEqual(['Sky', 'Places/Sea']);
    expect(sheet.querySelector('[aria-haspopup="dialog"]')).toBeNull();
    expect(document.querySelectorAll('[role="dialog"]')).toHaveLength(1);
  });

  it('adds a tag with one tap on its row', async () => {
    const { onAdd } = await render(true);
    const row = [...dialog().querySelectorAll<HTMLElement>('[role="option"]')].find(
      (option) => option.textContent.trim() === 'Sky'
    );
    row?.click();
    await settle();
    expect(onAdd).toHaveBeenCalledWith(expect.objectContaining({ id: 'tag-sky' }));
  });

  it('keeps the anchored combobox on desktop', async () => {
    layoutState.mode = 'desktop';
    await render(true);
    expect(document.querySelector('button[data-sheet-handle]')).toBeNull();
    expect(document.querySelector('[data-popover-content] input[role="combobox"]')).not.toBeNull();
  });
});
