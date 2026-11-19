import { restoreBrowse } from '#lib/browse/restore.ts';
import EditorRouteHarness from '#lib/testing/EditorRouteHarness.svelte';
import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const stores = vi.hoisted(() => ({
  editor: {
    assetId: null as string | null,
    previewUrl: null as string | null,
    previewFrame: null,
    saving: false,
    activeRetouchId: null as string | null,
    setActiveLayer: vi.fn(),
    setActiveMaskComponent: vi.fn(),
    load: vi.fn((_id: string) => Promise.resolve()),
    endGeometrySession: vi.fn(() => Promise.resolve()),
    unload: vi.fn()
  },
  browsing: { assets: [] as { id: string }[] }
}));

vi.mock('#lib/stores/editor.svelte.ts', () => ({ editor: stores.editor }));
vi.mock('#lib/stores/browsing.svelte.ts', () => ({ browsing: stores.browsing }));
vi.mock('#lib/browse/restore.ts', () => ({ restoreBrowse: vi.fn(() => Promise.resolve()) }));
vi.mock('#lib/editor/neighbours.ts', () => ({ warmNeighbours: vi.fn(() => false) }));
vi.mock('#lib/utils/route.ts', () => ({ returnPath: () => '/albums/x1' }));

type Harness = { show: (id: string) => void };

let target: HTMLDivElement;
let harness: Harness | null = null;

function render(): Harness {
  harness = mount(EditorRouteHarness, { target });
  flushSync();
  return harness;
}

function leave(): void {
  if (harness) void unmount(harness);
  harness = null;
  flushSync();
}

function beforeUnload(): Event {
  const event = new Event('beforeunload', { cancelable: true });
  window.dispatchEvent(event);
  return event;
}

beforeEach(() => {
  stores.editor.saving = false;
  stores.browsing.assets = [];
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  leave();
  target.remove();
  vi.clearAllMocks();
  vi.restoreAllMocks();
});

describe('runEditorRoute', () => {
  it('loads the photo once per id change', () => {
    const route = render();
    expect(stores.editor.load).toHaveBeenCalledExactlyOnceWith('a1');

    route.show('a1');
    flushSync();
    expect(stores.editor.load).toHaveBeenCalledOnce();

    route.show('b2');
    flushSync();
    expect(stores.editor.load).toHaveBeenCalledTimes(2);
    expect(stores.editor.load).toHaveBeenLastCalledWith('b2');
  });

  it('restores the browse list from the return path when none is loaded', () => {
    render();

    expect(restoreBrowse).toHaveBeenCalledExactlyOnceWith('/albums/x1', 'a1');
  });

  it('keeps a loaded browse list', () => {
    stores.browsing.assets = [{ id: 'a1' }];
    render();

    expect(restoreBrowse).not.toHaveBeenCalled();
  });

  it('unloads only after the geometry session has ended', async () => {
    let finish = (): void => {};
    stores.editor.endGeometrySession.mockReturnValueOnce(
      new Promise<void>((resolve) => {
        finish = resolve;
      })
    );
    render();

    leave();
    await Promise.resolve();
    expect(stores.editor.endGeometrySession).toHaveBeenCalledOnce();
    expect(stores.editor.unload).not.toHaveBeenCalled();

    finish();
    await vi.waitFor(() => expect(stores.editor.unload).toHaveBeenCalledOnce());
  });

  it('guards unload while saving with one listener for the route lifetime', () => {
    const add = vi.spyOn(window, 'addEventListener');
    const remove = vi.spyOn(window, 'removeEventListener');
    const route = render();
    route.show('b2');
    flushSync();

    const added = add.mock.calls.filter(([type]) => type === 'beforeunload');
    expect(added).toHaveLength(1);

    expect(beforeUnload().defaultPrevented).toBe(false);
    stores.editor.saving = true;
    expect(beforeUnload().defaultPrevented).toBe(true);

    leave();
    const removed = remove.mock.calls.filter(([type]) => type === 'beforeunload');
    expect(removed).toHaveLength(1);
    expect(removed[0]?.[1]).toBe(added[0]?.[1]);
  });
});
