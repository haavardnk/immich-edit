import type { ApplyPresetOptions } from '#lib/api/jobs.ts';
import { withLookAmount } from '#lib/edits/lookAmount.ts';
import { manifestToEdits } from '#lib/edits/manifest.ts';
import type { EditManifest, Edits } from '#lib/types/edits.ts';

export interface PresetPreview {
  manifest: EditManifest;
  opts: ApplyPresetOptions;
}

export function presetEdits(current: Edits, { manifest, opts }: PresetPreview): Edits {
  const incoming = withLookAmount(manifestToEdits(manifest), opts.amount);
  return {
    basic: incoming.basic,
    tone: incoming.tone,
    color: incoming.color,
    detail: incoming.detail,
    effects: incoming.effects,
    lens: incoming.lens,
    geometry: opts.includeGeometry ? incoming.geometry : current.geometry,
    masks: opts.includeMasks ? incoming.masks : current.masks,
    retouch: current.retouch,
    lens_blur: current.lens_blur
  };
}
