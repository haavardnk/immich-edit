import { beforeEach, describe, expect, it, vi } from 'vitest';
import { virtualCopies } from './virtualCopies.svelte';

const copies = vi.hoisted(() => ({ createVirtualCopies: vi.fn() }));

vi.mock('#lib/editor/copies.ts', () => copies);

describe('virtual copies', () => {
  beforeEach(() => {
    copies.createVirtualCopies.mockReset();
  });

  it('runs one batch at a time', async () => {
    let finish = (): void => {};
    copies.createVirtualCopies.mockReturnValue(new Promise<void>((done) => (finish = done)));

    const first = virtualCopies.create(['a']);
    expect(virtualCopies.busy).toBe(true);
    await virtualCopies.create(['b']);
    finish();
    await first;

    expect(copies.createVirtualCopies).toHaveBeenCalledExactlyOnceWith(['a']);
    expect(virtualCopies.busy).toBe(false);
  });

  it('frees the guard when a batch fails', async () => {
    copies.createVirtualCopies.mockRejectedValueOnce(new Error('offline'));

    await expect(virtualCopies.create(['a'])).rejects.toThrow('offline');

    expect(virtualCopies.busy).toBe(false);
  });

  it('skips an empty batch', async () => {
    await virtualCopies.create([]);

    expect(copies.createVirtualCopies).not.toHaveBeenCalled();
  });
});
