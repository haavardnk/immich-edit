import type { Preset } from '#lib/api/presets.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const stores = vi.hoisted(() => ({
  editor: {
    editsPreview: null as { owner: string } | null,
    previewPreset: vi.fn(),
    endPresetPreview: vi.fn(),
    applyPreset: vi.fn(() => Promise.resolve()),
    presetApplied: {
      current: null as { id: string; name: string; amount: number } | null,
      blend: vi.fn(),
      commitBlend: vi.fn(() => Promise.resolve())
    }
  }
}));

vi.mock('#lib/stores/editor.svelte.ts', () => ({ editor: stores.editor }));

const { ALL_PRESETS, FAVOURITE_PRESETS, presetStrip } =
  await import('#lib/stores/presetStrip.svelte.ts');
const { presets } = await import('#lib/stores/presets.svelte.ts');

function preset(id: string, name: string, group_name: string | null, favorite = false): Preset {
  return {
    id,
    name,
    group_name,
    manifest: { schema_version: 1, ops: {} },
    favorite,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z'
  };
}

const LOOSE = preset('1', 'Loose', null);
const PORTRA = preset('2', 'Portra', 'Film');
const MONO = preset('3', 'Mono', 'B&W');
const KODAK = preset('4', 'Kodák Gold', 'Film', true);

describe('presetStrip', () => {
  beforeEach(() => {
    presets.presets = [LOOSE, PORTRA, MONO];
    presetStrip.pick(ALL_PRESETS);
    presetStrip.applyOptions = { includeGeometry: false, includeMasks: false, amount: 100 };
    stores.editor.editsPreview = null;
    stores.editor.presetApplied.current = null;
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
    presetStrip.pick(':Film');
    expect(presetStrip.shown).toEqual([PORTRA]);
    presetStrip.pick(':');
    expect(presetStrip.shown).toEqual([LOOSE]);
  });

  it('falls back to All when the picked group disappears', () => {
    presetStrip.pick(':Film');
    presets.presets = [LOOSE, MONO];
    expect(presetStrip.group).toBe(ALL_PRESETS);
    expect(presetStrip.shown).toEqual([LOOSE, MONO]);
  });

  it('offers Favourites first only while a preset is starred', () => {
    expect(presetStrip.hasFavourites).toBe(false);
    presets.presets = [LOOSE, PORTRA, KODAK];
    expect(presetStrip.groupOptions.map((o) => o.key)).toEqual([
      FAVOURITE_PRESETS,
      ALL_PRESETS,
      ':Film',
      ':'
    ]);
    presetStrip.pick(FAVOURITE_PRESETS);
    expect(presetStrip.shown).toEqual([KODAK]);
    presets.presets = [LOOSE, PORTRA];
    expect(presetStrip.group).toBe(ALL_PRESETS);
  });

  it('shows group filters once there is more than one way to narrow the list', () => {
    presets.presets = [PORTRA];
    expect(presetStrip.filterable).toBe(false);
    presets.presets = [PORTRA, { ...PORTRA, id: '9', favorite: true }];
    expect(presetStrip.filterable).toBe(true);
  });

  it('searches names across every group and clears the search on a group pick', () => {
    presets.presets = [LOOSE, PORTRA, MONO, KODAK];
    presetStrip.pick(':B&W');
    presetStrip.query = ' kodak ';
    expect(presetStrip.searching).toBe(true);
    expect(presetStrip.shown).toEqual([KODAK]);
    presetStrip.query = 'o';
    expect(presetStrip.shown).toEqual([LOOSE, PORTRA, MONO, KODAK]);
    presetStrip.pick(':Film');
    expect(presetStrip.query).toBe('');
    expect(presetStrip.shown).toEqual([PORTRA, KODAK]);
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

  it('applies with the current options and returns Amount to 100% for the next apply', () => {
    presetStrip.applyOptions.amount = 80;
    presetStrip.preview(MONO);
    stores.editor.editsPreview = { owner: 'preset' };
    expect(presetStrip.pill?.tail).toBe('');
    presetStrip.apply(MONO);
    expect(stores.editor.applyPreset).toHaveBeenCalledWith(MONO, {
      includeGeometry: false,
      includeMasks: false,
      amount: 80
    });
    expect(presetStrip.applyOptions.amount).toBe(100);
    expect(presetStrip.pill).toBeNull();
    expect(presetStrip.appliedPill).toBeNull();
  });

  it('keeps Amount live on the applied preset and shows it in the pill', () => {
    stores.editor.presetApplied.current = { id: '2', name: 'Portra', amount: 65 };
    expect(presetStrip.appliedId).toBe('2');
    expect(presetStrip.amount).toBe(65);
    expect(presetStrip.appliedPill).toEqual({
      lead: 'Portra applied · Amount',
      label: '65%',
      tail: 'stays live until your next edit',
      mono: true
    });
    presetStrip.setAmount(40);
    expect(stores.editor.presetApplied.blend).toHaveBeenCalledWith(40);
    expect(presetStrip.applyOptions.amount).toBe(100);
    presetStrip.commitAmount();
    expect(stores.editor.presetApplied.commitBlend).toHaveBeenCalled();
  });

  it('previews and applies the next preset at the live amount', () => {
    stores.editor.presetApplied.current = { id: '2', name: 'Portra', amount: 65 };
    presetStrip.preview(MONO);
    expect(stores.editor.previewPreset).toHaveBeenCalledWith({
      manifest: MONO.manifest,
      opts: { includeGeometry: false, includeMasks: false, amount: 65 }
    });
    presetStrip.apply(MONO);
    expect(stores.editor.applyPreset).toHaveBeenCalledWith(MONO, {
      includeGeometry: false,
      includeMasks: false,
      amount: 65
    });
  });

  it('sets the amount for the next apply while nothing is applied', () => {
    presetStrip.setAmount(30);
    expect(stores.editor.presetApplied.blend).not.toHaveBeenCalled();
    expect(presetStrip.amount).toBe(30);
  });
});
