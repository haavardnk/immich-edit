import { cullState, type CullActions } from '#lib/browse/cullState.ts';
import LoupeActionRail from '#lib/components/browse/LoupeActionRail.svelte';
import BottomBar from '#lib/components/editor/BottomBar.svelte';
import { editor } from '#lib/stores/editor.svelte.ts';
import { session } from '#lib/stores/session.svelte.ts';
import { layout, layoutControls, type ControlRef, type Layout } from '#lib/testing/layoutParity.ts';
import type { AssetDetail, TagRef } from '#lib/types/asset.ts';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import CullControls from './CullControls.svelte';

vi.mock('$app/navigation', () => ({ afterNavigate: (): void => {}, goto: vi.fn() }));

vi.mock('$app/state', () => ({
  page: { url: new URL('http://localhost/photos'), params: {} }
}));

const label: TagRef = { id: 'l', name: 'red', value: 'immich-edit/label/red' };

function asset(ownerId: string): AssetDetail {
  return {
    id: 'a1',
    ownerId,
    originalFileName: 'IMG.ARW',
    type: 'IMAGE',
    originalMimeType: 'image/x-sony-arw',
    fileCreatedAt: null,
    updatedAt: null,
    checksum: null,
    isFavorite: true,
    exifInfo: null,
    tags: [label]
  };
}

const noop = (): void => {};

const actions: CullActions = { rate: noop, favorite: noop, reject: noop, label: noop };

const button = (name: string): ControlRef => ({ role: 'button', name });

function layouts(subject: AssetDetail): Layout<'body' | 'editor bottom bar' | 'loupe rail'>[] {
  const cull = cullState(subject);
  return [
    layout('body', CullControls, { ...cull, starSize: 16, actions }),
    layout(
      'editor bottom bar',
      BottomBar,
      {},
      {
        exclude: [
          button('Tags (Shift+T)'),
          button('Zoom'),
          button('Show presets'),
          button('No filmstrip available')
        ]
      }
    ),
    layout(
      'loupe rail',
      LoupeActionRail,
      {
        cull,
        actions,
        tags: subject.tags ?? [],
        multi: false,
        zoom: 1,
        fitZoom: 1,
        fitMode: true,
        onAddTag: () => Promise.resolve(),
        onRemoveTag: () => Promise.resolve(),
        onCreateTag: () => Promise.resolve(null),
        onZoom: noop,
        onFit: noop
      },
      { exclude: [button('Tags (Shift+T)'), button('Zoom'), button('Collapse filmstrip')] }
    )
  ];
}

beforeEach(() => {
  vi.stubGlobal(
    'fetch',
    vi.fn(() => Promise.reject(new Error('offline')))
  );
  session.set({ id: 'me', email: 'me@x', name: 'Me', is_admin: false, auth_kind: 'password' });
});

afterEach(() => {
  editor.asset = null;
  session.clear();
  vi.unstubAllGlobals();
});

describe('CullControls layout parity', () => {
  it.each([
    ['owned', 'me', false],
    ['shared', 'other', true]
  ])('exposes the same controls in every container (%s)', async (_, owner, locked) => {
    const subject = asset(owner);
    editor.asset = subject;
    const found = await layoutControls(layouts(subject));
    expect(found.body).toHaveLength(8);
    expect(found.body.every((control) => (control.disabled ?? false) === locked)).toBe(true);
    expect(found['editor bottom bar']).toEqual(found.body);
    expect(found['loupe rail']).toEqual(found.body);
  });
});
