import { neutralEdits, type Edits } from '#lib/types/edits.ts';
import { defaultRadial, makeLayer } from '#lib/types/masks.ts';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({ livePreview: vi.fn() }));

vi.mock('#lib/api/preview.ts', () => ({
  livePreview: mocks.livePreview,
  maskWeightPreview: (layerId: string) => ({ mask_weight: { layer_id: layerId } })
}));

import { MaskThumbs } from '#lib/stores/editor/maskThumbs.svelte.ts';
import { weightEdits } from '#lib/stores/editor/maskWeight.svelte.ts';

let created = 0;

function editsWith(names: string[]): Edits {
  return { ...neutralEdits(), masks: names.map((name) => makeLayer(name, defaultRadial())) };
}

beforeEach(() => {
  created = 0;
  vi.useFakeTimers();
  vi.stubGlobal('URL', {
    createObjectURL: vi.fn(() => `blob:${++created}`),
    revokeObjectURL: vi.fn()
  });
  mocks.livePreview.mockResolvedValue({ blob: new Blob(['x']) });
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
  mocks.livePreview.mockReset();
});

describe('MaskThumbs', () => {
  it('renders each layer once on its own lane after edits settle', async () => {
    const thumbs = new MaskThumbs();
    const edits = editsWith(['Sky', 'Subject']);
    thumbs.sync('a1', edits, true);
    thumbs.sync('a1', edits, true);
    await vi.runAllTimersAsync();

    expect(mocks.livePreview).toHaveBeenCalledTimes(2);
    expect(mocks.livePreview.mock.calls[0]?.[2]).toBe(96);
    expect(mocks.livePreview.mock.calls[0]?.[6]).toBe('mask_thumb');
    const [sky, subject] = edits.masks;
    expect(thumbs.urls[sky?.id ?? '']).toBe('blob:1');
    expect(thumbs.urls[subject?.id ?? '']).toBe('blob:2');
  });

  it('re-renders only the changed layer and drops deleted ones', async () => {
    const thumbs = new MaskThumbs();
    const edits = editsWith(['Sky', 'Subject']);
    thumbs.sync('a1', edits, true);
    await vi.runAllTimersAsync();
    const [sky, subject] = edits.masks;
    if (!sky || !subject) throw new Error('missing layers');

    thumbs.sync('a1', { ...edits, masks: [{ ...sky, invert: true }] }, true);
    await vi.runAllTimersAsync();

    expect(mocks.livePreview).toHaveBeenCalledTimes(3);
    expect(thumbs.urls[sky.id]).toBe('blob:3');
    expect(thumbs.urls[subject.id]).toBeUndefined();
    expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:1');
    expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:2');
  });

  it('waits for the editor and forgets the last photo on a new one', async () => {
    const thumbs = new MaskThumbs();
    const edits = editsWith(['Sky']);
    thumbs.sync('a1', edits, false);
    await vi.runAllTimersAsync();
    expect(mocks.livePreview).not.toHaveBeenCalled();

    thumbs.sync('a1', edits, true);
    await vi.runAllTimersAsync();
    thumbs.sync('a2', editsWith([]), true);
    expect(thumbs.urls).toEqual({});
    expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:1');
  });
});

describe('weightEdits', () => {
  it('isolates one shape as an added, enabled base without the layer invert', () => {
    const edits = editsWith(['Sky']);
    const layer = edits.masks[0];
    if (!layer) throw new Error('missing layer');
    const second = { ...layer.components[0]!, id: 'c2', mode: 'subtract' as const, enabled: false };
    edits.masks = [{ ...layer, invert: true, components: [...layer.components, second] }];

    const solo = weightEdits(edits, layer.id, 'c2');
    expect(solo?.masks).toHaveLength(1);
    expect(solo?.masks[0]?.invert).toBe(false);
    expect(solo?.masks[0]?.components).toEqual([{ ...second, mode: 'add', enabled: true }]);
    expect(weightEdits(edits, layer.id, 'missing')).toBeNull();
  });
});
