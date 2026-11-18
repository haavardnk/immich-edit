import { editor } from '#lib/stores/editor.svelte.ts';
import { trash } from '#lib/stores/trash.svelte.ts';
import type { AssetDetail } from '#lib/types/asset.ts';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { editorTrash } from './culling';

const route = vi.hoisted(() => ({
  page: { url: new URL('http://localhost/assets/a?from=%2Falbums%2Fx'), params: { id: 'a' } }
}));
const nav = vi.hoisted(() => ({ backToGrid: vi.fn(() => Promise.resolve()) }));

vi.mock('$app/state', () => route);
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('#lib/browse/backToGrid.ts', () => nav);

function answer(): (confirmed: boolean) => Promise<void> {
  let resolve = (_confirmed: boolean): void => undefined;
  const pending = new Promise<boolean>((done) => (resolve = done));
  vi.spyOn(trash, 'request').mockReturnValue(pending);
  return async (confirmed) => {
    resolve(confirmed);
    await pending;
    await Promise.resolve();
  };
}

beforeEach(() => {
  route.page.params.id = 'a';
  editor.asset = { id: 'a' } as AssetDetail;
});

afterEach(() => {
  editor.asset = null;
  nav.backToGrid.mockClear();
  vi.restoreAllMocks();
});

describe('editor trash', () => {
  it('trashes the routed asset and returns to the grid', async () => {
    const confirm = answer();
    editorTrash.run();
    expect(trash.request).toHaveBeenCalledWith([editor.asset]);
    await confirm(true);
    expect(nav.backToGrid).toHaveBeenCalledWith('a', '/albums/x', 'push');
  });

  it('stays on another asset opened while the dialog was up', async () => {
    const confirm = answer();
    editorTrash.run();
    route.page.params.id = 'b';
    await confirm(true);
    expect(nav.backToGrid).not.toHaveBeenCalled();
  });

  it('stays when the trash is cancelled', async () => {
    const confirm = answer();
    editorTrash.run();
    await confirm(false);
    expect(nav.backToGrid).not.toHaveBeenCalled();
  });

  it('is unavailable while the loaded asset lags the route', () => {
    route.page.params.id = 'b';
    expect(editorTrash.enabled()).toBe(false);
    route.page.params.id = 'a';
    expect(editorTrash.enabled()).toBe(true);
  });
});
