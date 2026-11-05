import { beforeEach, describe, expect, it, vi } from 'vitest';
import { getImmichCapabilities, type ImmichCapabilities } from '#lib/api/immich.ts';
import { immichCapabilities } from './immichCapabilities.svelte';

vi.mock('#lib/api/immich.ts', () => ({ getImmichCapabilities: vi.fn() }));

function capabilities(extra: Partial<ImmichCapabilities>): ImmichCapabilities {
  return {
    immich_version: '3.2.2',
    external_host: 'photos.example.net',
    structured_search: true,
    smart_search: true,
    ocr: true,
    trash: true,
    trash_days: 30,
    ...extra
  };
}

beforeEach(() => {
  immichCapabilities.externalHost = null;
  immichCapabilities.version = null;
});

describe('immich server identity', () => {
  it('carries the public Immich host and version', async () => {
    vi.mocked(getImmichCapabilities).mockResolvedValueOnce(capabilities({}));
    await immichCapabilities.load();
    expect(immichCapabilities.externalHost).toBe('photos.example.net');
    expect(immichCapabilities.version).toBe('3.2.2');
  });

  it('clears the host when Immich names no public domain', async () => {
    immichCapabilities.externalHost = 'old.example.net';
    vi.mocked(getImmichCapabilities).mockResolvedValueOnce(capabilities({ external_host: null }));
    await immichCapabilities.load();
    expect(immichCapabilities.externalHost).toBeNull();
  });
});
