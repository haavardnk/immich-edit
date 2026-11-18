import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  goto: vi.fn(() => Promise.resolve()),
  createCopy: vi.fn()
}));

vi.mock('$app/navigation', () => ({ goto: mocks.goto }));
vi.mock('#lib/api/copies.ts', () => ({ createCopy: mocks.createCopy }));

import { toasts } from '#lib/stores/toasts.svelte.ts';
import { createVirtualCopies, createVirtualCopy } from './copies';

describe('createVirtualCopy', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    mocks.goto.mockClear();
    mocks.createCopy.mockReset();
  });

  it.each([
    ['Copy 1', true, 'Created virtual copy "Copy 1"'],
    [null, true, 'Created virtual copy'],
    ['Copy 1', false, null]
  ])('name %s, navigate %s toasts %s', async (name, navigate, message) => {
    mocks.createCopy.mockResolvedValue({
      id: 'copy-1',
      source_asset_id: 'asset-1',
      name,
      created_at: '2026-10-21T00:00:00Z'
    });
    const push = vi.spyOn(toasts, 'push').mockImplementation(() => undefined);

    await createVirtualCopy('asset-1', { navigate });

    expect(mocks.goto).toHaveBeenCalledTimes(navigate ? 1 : 0);
    if (message) expect(push).toHaveBeenCalledWith('success', message, { ttlMs: 3000 });
    else expect(push).not.toHaveBeenCalled();
  });
});

describe('createVirtualCopies', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    mocks.goto.mockClear();
    mocks.createCopy.mockReset();
  });

  it('copies every photo without leaving the grid', async () => {
    mocks.createCopy.mockImplementation((id: string) =>
      Promise.resolve({ id: `${id}-copy`, source_asset_id: id, name: null, created_at: null })
    );
    const push = vi.spyOn(toasts, 'push').mockImplementation(() => undefined);

    await createVirtualCopies(['a', 'b']);

    expect(mocks.createCopy).toHaveBeenCalledTimes(2);
    expect(mocks.goto).not.toHaveBeenCalled();
    expect(push).toHaveBeenCalledWith('success', 'Created 2 virtual copies', { ttlMs: 4000 });
  });

  it('counts the copies Immich refused', async () => {
    mocks.createCopy.mockImplementation((id: string) =>
      id === 'b'
        ? Promise.reject(new Error('boom'))
        : Promise.resolve({ id: `${id}-copy`, source_asset_id: id, name: null, created_at: null })
    );
    const push = vi.spyOn(toasts, 'push').mockImplementation(() => undefined);

    await createVirtualCopies(['a', 'b', 'c']);

    expect(push).toHaveBeenCalledWith('warn', '1 of 3 failed', { ttlMs: 6000 });
  });
});
