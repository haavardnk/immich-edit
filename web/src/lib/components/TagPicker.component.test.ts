import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import TagPicker from './TagPicker.svelte';

type PickerProps = ComponentProps<typeof TagPicker>;

const Host: Component<{ component: Component<PickerProps>; props: PickerProps }> = ComponentHost;

const props: PickerProps = {
  tags: [],
  open: false,
  onToggle: () => {},
  onClose: () => {},
  onAdd: () => {},
  onRemove: () => {},
  onCreate: () => null
};

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

async function settle(): Promise<void> {
  for (let i = 0; i < 10; i++) {
    await new Promise((resolve) => setTimeout(resolve, 0));
    flushSync();
  }
}

beforeEach(() => {
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  target.remove();
  vi.unstubAllGlobals();
  vi.resetModules();
});

describe('TagPicker', () => {
  it('loads an empty tag library once', async () => {
    let calls = 0;
    const fetch = vi.fn<typeof globalThis.fetch>(() =>
      ++calls > 5
        ? new Promise<Response>(() => {})
        : Promise.resolve(new Response('[]', { status: 200 }))
    );
    vi.stubGlobal('fetch', fetch);
    component = mount(Host, { target, props: { component: TagPicker, props } });
    flushSync();
    await settle();
    const tagRequests = fetch.mock.calls.filter(([input]) => String(input).endsWith('/api/tags'));
    expect(tagRequests).toHaveLength(1);
  });
});
