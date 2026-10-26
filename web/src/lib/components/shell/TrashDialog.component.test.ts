import { immichCapabilities } from '$lib/stores/immichCapabilities.svelte';
import { trash } from '$lib/stores/trash.svelte';
import ComponentHost from '$lib/testing/ComponentHost.svelte';
import { flushSync, mount, unmount, type Component } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import TrashDialog from './TrashDialog.svelte';

const Host: Component<{ component: Component; props: Record<string, never> }> = ComponentHost;

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

async function settle(): Promise<void> {
  for (let i = 0; i < 5; i++) {
    await new Promise((resolve) => setTimeout(resolve, 0));
    flushSync();
  }
}

function button(name: string): HTMLButtonElement | undefined {
  return [...document.querySelectorAll<HTMLButtonElement>('button')].find(
    (b) => b.textContent?.trim() === name
  );
}

beforeEach(() => {
  immichCapabilities.trashDays = 30;
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  trash.plan = null;
  target.remove();
  vi.restoreAllMocks();
});

describe('TrashDialog', () => {
  it('stays closed without a pending request', async () => {
    component = mount(Host, { target, props: { component: TrashDialog, props: {} } });
    await settle();
    expect(document.querySelector('[role="dialog"]')).toBeNull();
  });

  it('says what happens and answers the pending request', async () => {
    const confirm = vi.spyOn(trash, 'confirm');
    const cancel = vi.spyOn(trash, 'cancel');
    trash.plan = { masters: ['a'], copies: [], versions: [], skipped: 0 };
    component = mount(Host, { target, props: { component: TrashDialog, props: {} } });
    await settle();

    const dialog = document.querySelector('[role="dialog"]');
    expect(dialog?.textContent).toContain('Move to Immich trash');
    expect(dialog?.textContent).toContain('1 photo moves to the Immich trash');
    expect(dialog?.textContent).toContain('after 30 days');
    expect(document.activeElement).toBe(button('Move to trash'));

    button('Move to trash')?.click();
    button('Cancel')?.click();
    expect(confirm).toHaveBeenCalledOnce();
    expect(cancel).toHaveBeenCalledOnce();
  });
});
