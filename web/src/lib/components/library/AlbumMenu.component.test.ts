import { session } from '#lib/stores/session.svelte.ts';
import { album } from '#lib/testing/albums.ts';
import ComponentHost from '#lib/testing/ComponentHost.svelte';
import { settle } from '#lib/testing/layoutParity.ts';
import type { AlbumMember } from '#lib/types/album.ts';
import { mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import AlbumMenu from './AlbumMenu.svelte';

const h = vi.hoisted(() => ({ run: vi.fn(() => Promise.resolve()) }));

vi.mock('#lib/stores/albumActions.svelte.ts', () => ({ albumActions: { run: h.run } }));

type Props = ComponentProps<typeof AlbumMenu>;

const Host: Component<{ component: Component<Props>; props: Props }> = ComponentHost;

const ME = 'user-me';
const ANNE: AlbumMember = { userId: 'user-anne', name: 'Anne', role: 'owner' };

let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

async function render(props: Props): Promise<string[]> {
  component = mount(Host, { target, props: { component: AlbumMenu, props } });
  await settle();
  return [...target.querySelectorAll('button')].map((b) => b.textContent?.trim() ?? '');
}

function shared(role: AlbumMember['role']) {
  return album('a', {
    albumName: 'Family',
    ownerId: ANNE.userId,
    members: [ANNE, { userId: ME, name: 'Me', role }]
  });
}

beforeEach(() => {
  session.set({ id: ME, email: 'me@x', name: 'Me', is_admin: false, auth_kind: 'password' });
  target = document.createElement('div');
  document.body.appendChild(target);
  h.run.mockClear();
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  session.clear();
  target.remove();
});

describe('AlbumMenu', () => {
  it('gives an owner rename, description and delete', async () => {
    const own = album('a', { ownerId: ME });
    expect(await render({ album: own, place: 'page', variant: 'popover' })).toEqual([
      'Rename',
      'Edit description',
      'Delete album'
    ]);
  });

  it('lets an editor edit and leave, and a viewer only leave', async () => {
    expect(await render({ album: shared('editor'), place: 'page', variant: 'popover' })).toEqual([
      'Rename',
      'Edit description',
      'Leave album'
    ]);
    void unmount(component as Record<string, unknown>);
    component = null;
    expect(await render({ album: shared('viewer'), place: 'list', variant: 'popover' })).toEqual([
      'Open album',
      'Leave album'
    ]);
  });

  it('closes its container and runs the picked action on the album', async () => {
    const onrun = vi.fn();
    const viewer = shared('viewer');
    await render({ album: viewer, place: 'list', variant: 'popover', onrun });
    const leave = [...document.querySelectorAll('button')].find(
      (b) => b.textContent?.trim() === 'Leave album'
    );
    leave?.click();
    expect(onrun).toHaveBeenCalledOnce();
    expect(h.run).toHaveBeenCalledWith('leave', viewer);
  });
});
