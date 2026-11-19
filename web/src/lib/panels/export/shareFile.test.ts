import { afterEach, describe, expect, it, vi } from 'vitest';
import { baseOptions, defaultExportForm } from './settings';
import { saveOnly, shareFile, shareSupport } from './shareFile';

function file(format: 'jpeg' | 'heic' | 'jxl' = 'jpeg') {
  return {
    blob: new Blob(['x']),
    name: `IMG_0001_edit.${format}`,
    options: { ...baseOptions(defaultExportForm()), format }
  };
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('share support', () => {
  it('is off without the Web Share API', () => {
    vi.stubGlobal('navigator', {});
    expect(shareSupport(file())).toEqual({ shareable: false, reason: null });
    expect(saveOnly('heic')).toBe(false);
  });

  it('marks the formats a sharing browser refuses as save only', () => {
    vi.stubGlobal('navigator', { canShare: () => true });
    expect(saveOnly('heic')).toBe(true);
    expect(saveOnly('jxl')).toBe(true);
    expect(saveOnly('jpeg')).toBe(false);
  });

  it('asks the browser with a typed file', () => {
    const canShare = vi.fn(() => true);
    vi.stubGlobal('navigator', { canShare });
    expect(shareSupport(file())).toEqual({ shareable: true, reason: null });
    const [data] = canShare.mock.calls[0] as unknown as [{ files: File[] }];
    expect(data.files[0]?.type).toBe('image/jpeg');
    expect(data.files[0]?.name).toBe('IMG_0001_edit.jpeg');
  });

  it('says why HEIC and JPEG XL cannot be shared', () => {
    vi.stubGlobal('navigator', { canShare: () => false });
    expect(shareSupport(file('heic')).reason).toBe("This browser can't share HEIC files.");
    expect(shareSupport(file('jxl')).reason).toBe("This browser can't share JPEG XL files.");
    expect(shareSupport(file('jpeg'))).toEqual({ shareable: false, reason: null });
  });
});

describe('share file', () => {
  it('treats a dismissed share sheet as no share', async () => {
    vi.stubGlobal('navigator', {
      share: vi.fn(() => Promise.reject(new DOMException('cancelled', 'AbortError')))
    });
    await expect(shareFile(file())).resolves.toBe(false);
  });

  it('passes other failures on', async () => {
    vi.stubGlobal('navigator', { share: vi.fn(() => Promise.reject(new Error('too big'))) });
    await expect(shareFile(file())).rejects.toThrow('too big');
  });
});
