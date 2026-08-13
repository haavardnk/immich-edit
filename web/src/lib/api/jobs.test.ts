import { afterEach, describe, expect, it, vi } from 'vitest';
import { createImmichExportJob, createZipExportJob } from './jobs';
import type { ExportOptions, ImmichExportOptions } from './export';

const base: ExportOptions = {
  format: 'jpeg',
  quality: 90,
  includeExif: true,
  bitDepth: '8',
  pngCompression: 'default',
  tiffCompression: 'lzw',
  lossless: false,
  colorSpace: 'displayp3'
};

const immich: ImmichExportOptions = {
  ...base,
  albumIds: [],
  tagIds: [],
  favorite: false,
  stackWithOriginal: false,
  stackPrimary: 'edited',
  filenameSuffix: '_edit'
};

function stubFetch(): { body: () => Record<string, unknown> } {
  let sent = '';
  vi.stubGlobal(
    'fetch',
    vi.fn(async (_input: RequestInfo, init?: RequestInit) => {
      sent = String(init?.body ?? '');
      return new Response('{}', { headers: { 'content-type': 'application/json' } });
    })
  );
  return { body: () => JSON.parse(sent) };
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('export jobs', () => {
  it.each([
    ['zip', () => createZipExportJob({ assetIds: ['a'] }, base, '_edit')],
    ['immich', () => createImmichExportJob({ search: { tagIds: ['t'] } }, immich)]
  ])('sends the chosen color space for %s exports', async (_kind, run) => {
    const req = stubFetch();
    await run();
    expect((req.body().params as Record<string, unknown>).color_space).toBe('displayp3');
  });

  it('sends a search target instead of asset ids', async () => {
    const req = stubFetch();
    await createZipExportJob({ search: { tagIds: ['t'] } }, base, '_edit');
    const body = req.body();
    expect(body.target).toEqual({ search: { tagIds: ['t'] } });
    expect(body.asset_ids).toBeUndefined();
  });
});
