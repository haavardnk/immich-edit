import { albumActions } from '#lib/stores/albumActions.svelte.ts';
import { session } from '#lib/stores/session.svelte.ts';
import { album } from '#lib/testing/albums.ts';
import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { flushSync, mount, unmount, type Component } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import AlbumConfirmDialog from './AlbumConfirmDialog.svelte';

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/state', () => ({ page: { url: new URL('http://localhost/albums') } }));

const Host: Component<{ component: Component; props: Record<string, never> }> = ComponentHost;

const ME = 'user-me';

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
  session.set({ id: ME, email: 'me@x', name: 'Me', is_admin: false, auth_kind: 'password' });
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  albumActions.confirming = null;
  session.clear();
  target.remove();
  vi.restoreAllMocks();
});

describe('AlbumConfirmDialog', () => {
  it('stays closed without a pending request', async () => {
    component = mount(Host, { target, props: { component: AlbumConfirmDialog, props: {} } });
    await settle();
    expect(document.querySelector('[role="dialog"]')).toBeNull();
  });

  it('names the album in bold and answers the request', async () => {
    const confirm = vi.spyOn(albumActions, 'confirm').mockResolvedValue();
    const cancel = vi.spyOn(albumActions, 'cancel');
    albumActions.confirming = {
      kind: 'delete',
      album: album('a', { albumName: 'Norway', assetCount: 2, ownerId: ME })
    };
    component = mount(Host, { target, props: { component: AlbumConfirmDialog, props: {} } });
    await settle();

    const dialog = document.querySelector('[role="dialog"]');
    expect(dialog?.querySelector('b')?.textContent).toBe('Norway');
    expect(dialog?.textContent).toContain(
      'Immich deletes the album Norway. Its 2 photos stay in your library'
    );
    expect(document.activeElement).toBe(button('Delete album'));

    button('Delete album')?.click();
    button('Cancel')?.click();
    expect(confirm).toHaveBeenCalledOnce();
    expect(cancel).toHaveBeenCalledOnce();
  });
});
