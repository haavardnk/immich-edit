import type { ApplyPresetOptions } from '$lib/api/jobs';
import { withLookAmount } from '$lib/edits/lookAmount';
import { manifestToEdits } from '$lib/edits/manifest';
import type { EditManifest, Edits } from '$lib/types/edits';

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
