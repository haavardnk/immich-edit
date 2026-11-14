import type { PillText } from '#lib/editor/pill.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import { formatBrushSize } from '#lib/utils/brushSize.ts';
import { retouchValue } from '#lib/editor/retouch/settings.ts';
import { modeName } from '#lib/editor/retouch/text.ts';

export const SECOND_FINGER_MS = 120;

type PaintInputType = 'touch' | 'pen';

class PaintInput {
  penSeen = $state(false);
  painting = $state<PaintInputType | null>(null);
  sizing = $state(false);

  get brushActive(): boolean {
    return ui.editorTab === 'masks' && editor.activeMaskComponent()?.kind.kind === 'brush';
  }

  get retouchActive(): boolean {
    return ui.editorTab === 'retouch';
  }

  get fingersPaint(): boolean {
    return (this.brushActive || this.retouchActive) && !this.penSeen;
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
    const [tool, pen, size] = this.retouchActive ? this.#retouchTool() : this.#brushTool();
    return {
      lead: this.painting === 'pen' ? `Pencil · ${pen}` : tool,
      label: formatBrushSize(size, editor.meta),
      tail: '',
      mono: true
    };
  }

  #brushTool(): [string, string, number] {
    const erase = editor.brushTool.mode === 'erase';
    return [erase ? 'Erase' : 'Brush', erase ? 'Erase' : 'Paint', editor.brushTool.size];
  }

  #retouchTool(): [string, string, number] {
    const mode = modeName(editor.retouchTool.mode);
    return [mode, mode, this.painting ? editor.retouchTool.size : retouchValue('size')];
  }
}

export const paintInput = new PaintInput();
