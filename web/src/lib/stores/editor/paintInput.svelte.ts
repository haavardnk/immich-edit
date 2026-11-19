import type { PillText } from '#lib/editor/pill.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import { formatBrushSize } from '#lib/utils/brushSize.ts';

export const SECOND_FINGER_MS = 120;

type PaintInputType = 'touch' | 'pen';

class PaintInput {
  penSeen = $state(false);
  painting = $state<PaintInputType | null>(null);
  sizing = $state(false);

  get brushActive(): boolean {
    return ui.editorTab === 'masks' && editor.activeMaskComponent()?.kind.kind === 'brush';
  }

  get fingersPaint(): boolean {
    return this.brushActive && !this.penSeen;
  }

  paints(pointerType: string): boolean {
    if (pointerType === 'pen') this.penSeen = true;
    return pointerType !== 'touch' || !this.penSeen;
  }

  begin(pointerType: string): void {
    this.painting = pointerType === 'touch' || pointerType === 'pen' ? pointerType : null;
  }

  end = (): void => {
    this.painting = null;
  };

  reset = (): void => {
    this.penSeen = false;
    this.painting = null;
    this.sizing = false;
  };

  get pill(): PillText | null {
    if (!this.painting && !this.sizing) return null;
    const erase = editor.brushTool.mode === 'erase';
    const lead =
      this.painting === 'pen' ? `Pencil · ${erase ? 'Erase' : 'Paint'}` : erase ? 'Erase' : 'Brush';
    return {
      lead,
      label: formatBrushSize(editor.brushTool.size, editor.meta),
      tail: '',
      mono: true
    };
  }
}

export const paintInput = new PaintInput();
