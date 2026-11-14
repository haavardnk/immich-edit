import type { LutMeta } from '#lib/api/luts.ts';
import { layout } from '#lib/device/layout.svelte.ts';
import { withLut } from '#lib/editor/lutPreview.ts';
import type { PillText } from '#lib/editor/pill.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import type { Edits } from '#lib/types/edits.ts';
import { mdiEyeOutline } from '@mdi/js';

type PreviewedLut = Pick<LutMeta, 'id' | 'name'>;

class LutPreviewStore {
  private previewing = $state<PreviewedLut | null>(null);

  get id(): string | null {
    return this.active?.id ?? null;
  }

  get pill(): PillText | null {
    const active = this.active;
    if (!active) return null;
    return {
      lead: 'Previewing LUT ·',
      label: active.name,
      tail: '',
      mono: false,
      icon: mdiEyeOutline,
      hint: layout.mode === 'touch' ? 'Release to return' : 'Click to apply · move away to restore'
    };
  }

  show = (lut: PreviewedLut): void => {
    if (lut.id === editor.edits.color.lut_3d.lut_id) {
      this.end();
      return;
    }
    this.previewing = { id: lut.id, name: lut.name };
    editor.previewEdits('lut', withLut($state.snapshot(editor.edits) as Edits, lut.id));
  };

  end = (): void => {
    if (!this.previewing) return;
    this.previewing = null;
    editor.endEditsPreview('lut');
  };

  private get active(): PreviewedLut | null {
    return editor.editsPreview?.owner === 'lut' ? this.previewing : null;
  }
}

export const lutPreview = new LutPreviewStore();
