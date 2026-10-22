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

  it('follow the per-asset revision so a finished bake changes the URL', async () => {
    const pending = [{ id: 'a1', hash: 'h1', updated_at: 'u', revision: 'd1-l1-dn' }];
    mockedList.mockResolvedValueOnce({ render_revision: 'd1-l1', items: pending });
    await editedThumbs.refresh();
    const before = assetThumbUrl('a1');

    const ready = [{ id: 'a1', hash: 'h1', updated_at: 'u', revision: 'd1-l1-dnab' }];
    mockedList.mockResolvedValueOnce({ render_revision: 'd1-l1', items: ready });
    await editedThumbs.refresh();
    const after = assetThumbUrl('a1');

    expect(before).toContain('r=d1-l1-dn&');
    expect(after).toContain('r=d1-l1-dnab&');
  });

  it('fall back to the render revision without a per-asset one or after a save', async () => {
    const items = [
      { id: 'a1', hash: 'h1', updated_at: 'u', revision: 'd1-l1-dnab' },
      { id: 'a2', hash: 'h2', updated_at: 'u' }
    ];
    mockedList.mockResolvedValueOnce({ render_revision: 'd1-l1', items });
    await editedThumbs.refresh();
    expect(assetThumbUrl('a2')).toContain('r=d1-l1&');

    editedThumbs.upsert('a1', 'h3', 'u2');
    expect(assetThumbUrl('a1')).toContain('h=h3&r=d1-l1&');
  });
});
