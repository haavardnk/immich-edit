import { describe, expect, it, vi } from 'vitest';
import { assetThumbUrl } from '$lib/api/assets';
import { listEditedAssets } from '$lib/api/edits';
import { editedThumbs } from './editedThumbs.svelte';

vi.mock('$lib/api/edits', () => ({ listEditedAssets: vi.fn() }));

const mockedList = vi.mocked(listEditedAssets);

describe('edited thumbnail URLs', () => {
  it('change when the server render revision changes', async () => {
    const items = [{ id: 'a1', hash: 'h1', updated_at: '2026-01-01T00:00:00Z' }];
    mockedList.mockResolvedValueOnce({ render_revision: 'd1-l1', items });
    await editedThumbs.refresh();
    const before = assetThumbUrl('a1');

    mockedList.mockResolvedValueOnce({ render_revision: 'd1-l0', items });
    await editedThumbs.refresh();
    const after = assetThumbUrl('a1');

    expect(before).toContain('h=h1');
    expect(after).toContain('h=h1');
    expect(after).not.toBe(before);
  });
});
