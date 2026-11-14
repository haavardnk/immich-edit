import { presetEdits } from '#lib/edits/applyPreset.ts';
import { editsToManifest } from '#lib/edits/manifest.ts';
import type { AssetDetail } from '#lib/types/asset.ts';
import { neutralEdits, type EditRecord, type Edits } from '#lib/types/edits.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  getAsset: vi.fn(),
  getEdits: vi.fn(),
  putEdits: vi.fn(),
  livePreview: vi.fn(() => new Promise<never>(() => undefined))
}));

vi.mock('#lib/api/assets.ts', () => ({ getAsset: mocks.getAsset }));
vi.mock('#lib/api/edits.ts', () => ({
  getEdits: mocks.getEdits,
  putEdits: mocks.putEdits,
  deleteEdits: vi.fn(() => Promise.resolve()),
  restoreEdits: vi.fn(),
  listEditHistory: vi.fn(() => Promise.resolve([])),
  autoEdits: vi.fn()
}));
vi.mock('#lib/api/lensProfile.ts', () => ({ getLensProfile: vi.fn(() => Promise.resolve(null)) }));
vi.mock('#lib/api/preview.ts', async (original) => ({
  ...(await original<typeof import('#lib/api/preview.ts')>()),
  livePreview: mocks.livePreview
}));

import { editor } from '#lib/stores/editor.svelte.ts';
import { PresetApplied, type PresetSteps } from '#lib/stores/editor/presetApplied.svelte.ts';

function asset(): AssetDetail {
  return {
    id: 'asset-1',
    originalFileName: 'photo.raw',
    type: 'IMAGE',
    originalMimeType: 'image/x-raw',
    fileCreatedAt: null,
    updatedAt: null,
    checksum: null,
    isFavorite: false,
    exifInfo: null,
    tags: []
  };
}

function record(edits: Edits): EditRecord {
  return {
    schema_version: 1,
    asset_id: 'asset-1',
    renderer_version: 'test',
    manifest: editsToManifest(edits),
    hash: 'hash-0',
    updated_at: '2026-09-01T00:00:00Z',
    immich_updated_at: null,
    immich_checksum: null
  };
}

function before(): Edits {
  const edits = neutralEdits();
  edits.basic.exposure_ev = 1;
  return edits;
}

function look(): Edits {
  const edits = neutralEdits();
  edits.basic.contrast = 40;
  return edits;
}

const PORTRA = { id: 'portra', name: 'Portra', manifest: editsToManifest(look()) };

const FULL = { includeGeometry: false, includeMasks: false, amount: 100 };

function actions(): unknown[] {
  return mocks.putEdits.mock.calls.map((call) => call[3]);
}

describe('applied preset', () => {
  beforeEach(async () => {
    editor.unload();
    mocks.putEdits.mockReset();
    mocks.putEdits.mockImplementation((_id: string, edits: Edits) =>
      Promise.resolve({ ...record(edits), hash: `hash-${mocks.putEdits.mock.calls.length}` })
    );
    mocks.getAsset.mockResolvedValue(asset());
    mocks.getEdits.mockResolvedValue(record(before()));
    await editor.load('asset-1');
  });

  it('marks the preset as applied until the edits change', async () => {
    await editor.applyPreset(PORTRA, FULL);
    expect(editor.presetApplied.current).toEqual({ id: 'portra', name: 'Portra', amount: 100 });
    expect(actions()).toEqual(['Preset: Portra']);

    editor.edits.basic.exposure_ev = 2;
    expect(editor.presetApplied.current).toBeNull();
    editor.edits.basic.exposure_ev = 0;
    await editor.onCommit('Exposure');
    expect(editor.presetApplied.current).toBeNull();
  });

  it('re-blends Amount from the edits before the preset without adding a step', async () => {
    await editor.applyPreset(PORTRA, FULL);
    expect(editor.edits.basic.contrast).toBe(40);

    editor.presetApplied.blend(50);
    expect(editor.edits.basic.contrast).toBe(20);
    editor.presetApplied.blend(65);
    expect(editor.edits.basic.contrast).toBe(26);
    expect(editor.presetApplied.current?.amount).toBe(65);
    expect(mocks.putEdits).toHaveBeenCalledTimes(1);

    await editor.presetApplied.commitBlend();
    expect(actions()).toEqual(['Preset: Portra', 'Preset: Portra (65%)']);
    expect(editor.presetApplied.current?.amount).toBe(65);

    editor.undo();
    expect(editor.edits.basic.contrast).toBe(0);
    expect(editor.edits.basic.exposure_ev).toBe(1);
    expect(editor.canUndo).toBe(false);
  });

  it('stays cleared after an undo and redo', async () => {
    await editor.applyPreset(PORTRA, FULL);
    editor.undo();
    expect(editor.presetApplied.current).toBeNull();
    editor.redo();
    expect(editor.edits.basic.contrast).toBe(40);
    expect(editor.presetApplied.current).toBeNull();
    editor.presetApplied.blend(10);
    expect(editor.edits.basic.contrast).toBe(40);
  });

  it('forgets the applied preset when the asset unloads', async () => {
    await editor.applyPreset(PORTRA, FULL);
    const applied = JSON.parse(JSON.stringify(editor.edits)) as Edits;
    editor.unload();
    mocks.getEdits.mockResolvedValue(record(applied));
    await editor.load('asset-1');
    expect(editor.edits.basic.contrast).toBe(40);
    expect(editor.presetApplied.current).toBeNull();
  });

  it('does not arm when the preset equals the step already on top', async () => {
    editor.edits = presetEdits(before(), { manifest: PORTRA.manifest, opts: FULL });
    await editor.onCommit('Paste');

    await editor.applyPreset(PORTRA, FULL);
    expect(editor.presetApplied.current).toBeNull();
    editor.presetApplied.blend(50);
    await editor.presetApplied.commitBlend();
    expect(editor.edits.basic.contrast).toBe(40);

    editor.undo();
    expect(editor.edits.basic.contrast).toBe(0);
    expect(editor.edits.basic.exposure_ev).toBe(1);
    expect(editor.canUndo).toBe(false);
  });
});

describe('applied preset amend guard', () => {
  function harness(): {
    applied: PresetApplied;
    steps: PresetSteps & { topStep: number | null };
    commits: Array<string | undefined>;
    saves: string[];
  } {
    const commits: Array<string | undefined> = [];
    const saves: string[] = [];
    const steps = {
      topStep: 1 as number | null,
      replaceStep: vi.fn((id: number) => id === steps.topStep)
    };
    const ctx = {
      initialised: true,
      edits: before(),
      onLive: () => undefined,
      onCommit: (action?: string) => {
        commits.push(action);
        steps.topStep = (steps.topStep ?? 0) + 1;
        return Promise.resolve();
      }
    };
    const applied = new PresetApplied(ctx, {
      steps,
      save: (_edits, action) => {
        saves.push(action);
        return Promise.resolve();
      }
    });
    return { applied, steps, commits, saves };
  }

  it('replaces the step the preset pushed while it is still on top', async () => {
    const { applied, steps, saves } = harness();
    await applied.apply(PORTRA, FULL);
    applied.blend(50);
    await applied.commitBlend();
    expect(steps.replaceStep).toHaveBeenCalledWith(2, expect.anything());
    expect(saves).toEqual(['Preset: Portra (50%)']);
  });

  it('commits a new step instead when another step took the top', async () => {
    const { applied, steps, commits, saves } = harness();
    await applied.apply(PORTRA, FULL);
    steps.topStep = 7;
    applied.blend(50);
    await applied.commitBlend();
    expect(saves).toEqual([]);
    expect(commits).toEqual(['Preset: Portra', 'Preset: Portra (50%)']);
  });
});
