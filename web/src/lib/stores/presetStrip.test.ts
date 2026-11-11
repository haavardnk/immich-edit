import type { Preset } from '#lib/api/presets.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const stores = vi.hoisted(() => ({
  editor: {
    editsPreview: null as { owner: string } | null,
    previewPreset: vi.fn(),
    endPresetPreview: vi.fn(),
    applyPreset: vi.fn(() => Promise.resolve())
  }
}));

vi.mock('#lib/stores/editor.svelte.ts', () => ({ editor: stores.editor }));

const { presetStrip } = await import('#lib/stores/presetStrip.svelte.ts');
const { presets } = await import('#lib/stores/presets.svelte.ts');

function preset(id: string, name: string, group_name: string | null): Preset {
  return {
    id,
    name,
    group_name,
    manifest: { schema_version: 1, ops: {} },
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z'
  } as Preset;
}

const LOOSE = preset('1', 'Loose', null);
const PORTRA = preset('2', 'Portra', 'Film');
const MONO = preset('3', 'Mono', 'B&W');

describe('presetStrip', () => {
  beforeEach(() => {
    presets.presets = [LOOSE, PORTRA, MONO];
    presetStrip.pick(null);
    stores.editor.editsPreview = null;
    vi.clearAllMocks();
  });

  it('lists groups A to Z with Ungrouped last and filters by the picked one', () => {
    expect(presetStrip.groupOptions.map((o) => o.label)).toEqual([
      'All',
      'B&W',
      'Film',
      'Ungrouped'
    ]);
    expect(presetStrip.shown).toEqual([LOOSE, PORTRA, MONO]);
    presetStrip.pick('Film');
    expect(presetStrip.shown).toEqual([PORTRA]);
    presetStrip.pick('');
    expect(presetStrip.shown).toEqual([LOOSE]);
  });

  it('falls back to All when the picked group disappears', () => {
    presetStrip.pick('Film');
    presets.presets = [LOOSE, MONO];
    expect(presetStrip.group).toBeNull();
    expect(presetStrip.shown).toEqual([LOOSE, MONO]);
  });

  it('names a held preview in the pill until the preview ends', () => {
    presetStrip.preview(PORTRA, true);
    expect(stores.editor.previewPreset).toHaveBeenCalledWith({
      manifest: PORTRA.manifest,
      opts: presetStrip.applyOptions
    });
    expect(presetStrip.pill).toBeNull();
    stores.editor.editsPreview = { owner: 'preset' };
    expect(presetStrip.pill).toEqual({
      lead: 'Previewing',
      label: 'Portra',
      tail: '· release to cancel',
      mono: false
    });
    presetStrip.endPreview();
    expect(stores.editor.endPresetPreview).toHaveBeenCalled();
    expect(presetStrip.pill).toBeNull();
  });

  it('applies with the current options and drops the preview name', () => {
    presetStrip.applyOptions.amount = 80;
    presetStrip.preview(MONO);
    stores.editor.editsPreview = { owner: 'preset' };
    expect(presetStrip.pill?.tail).toBe('');
    presetStrip.apply(MONO);
    expect(stores.editor.applyPreset).toHaveBeenCalledWith(
      MONO.manifest,
      { includeGeometry: false, includeMasks: false, amount: 80 },
      'Mono'
    );
    expect(presetStrip.pill).toBeNull();
  });
});
