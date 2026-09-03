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
    ['zip', () => createZipExportJob(['a'], base, '_edit')],
    ['immich', () => createImmichExportJob(['a'], immich)]
  ])('sends the chosen color space for %s exports', async (_kind, run) => {
    const req = stubFetch();
    await run();
    expect((req.body().params as Record<string, unknown>).color_space).toBe('displayp3');
  });

  it('sends concrete asset ids', async () => {
    const req = stubFetch();
    await createZipExportJob(['a', 'b'], base, '_edit');
    const body = req.body();
    expect(body.asset_ids).toEqual(['a', 'b']);
  });
});
