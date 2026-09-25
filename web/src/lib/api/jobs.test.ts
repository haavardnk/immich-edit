import { afterEach, describe, expect, it, vi } from 'vitest';
import { createImmichExportJob, createZipExportJob } from './jobs';
import type { ExportOptions, ImmichExportOptions } from './export';

const base: ExportOptions = {
  format: 'jpeg',
  quality: 90,
  metadata: 'all',
  bitDepth: '8',
  pngCompression: 'default',
  tiffCompression: 'lzw',
  lossless: false,
  colorSpace: 'displayp3',
  filenameTemplate: '{date}_{seq}',
  resize: { mode: 'dimensions', width: 2048, height: 2048, enlarge: false },
  sharpen: { media: 'glossy', amount: 'high', ppi: 360 },
  watermark: { id: 'wm', size: 20, opacity: 80, anchor: 'bottom', inset: 3 }
};

const immich: ImmichExportOptions = {
  ...base,
  albumIds: [],
  tagIds: [],
  favorite: false,
  stackWithOriginal: false,
  stackPrimary: 'edited'
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
    ['zip', () => createZipExportJob(['a'], base)],
    ['immich', () => createImmichExportJob(['a'], immich)]
  ])('sends the chosen color space and filename template for %s exports', async (_kind, run) => {
    const req = stubFetch();
    await run();
    const params = req.body().params as Record<string, unknown>;
    expect(params.color_space).toBe('displayp3');
    expect(params.filename_template).toBe('{date}_{seq}');
    expect(params).toMatchObject({
      resize_mode: 'dimensions',
      resize_width: 2048,
      resize_height: 2048,
      resize_enlarge: false,
      output_sharpen_media: 'glossy',
      output_sharpen_amount: 'high',
      output_sharpen_ppi: 360,
      watermark_id: 'wm',
      watermark_size: 0.2,
      watermark_opacity: 0.8,
      watermark_anchor: 'bottom',
      watermark_inset: 0.03
    });
    expect(params).not.toHaveProperty('filename_suffix');
  });

  it('sends concrete asset ids', async () => {
    const req = stubFetch();
    await createZipExportJob(['a', 'b'], base);
    const body = req.body();
    expect(body.asset_ids).toEqual(['a', 'b']);
  });
});
