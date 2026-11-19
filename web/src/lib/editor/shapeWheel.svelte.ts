import { shapeExtentLabel, shapeFeatherLabel, wheeledKind } from '#lib/editor/maskWheel.ts';
import type { PillText } from '#lib/editor/pill.ts';
import { held } from '#lib/input/modifiers.svelte.ts';
import { keysFor } from '#lib/shortcuts/labels.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import { wheelNotches } from '#lib/utils/brushSize.ts';

const SETTLE_MS = 400;
const FLASH_MS = 1200;

class ShapeWheel {
  pill = $state<PillText | null>(null);
  #settle: ReturnType<typeof setTimeout> | null = null;
  #flash: ReturnType<typeof setTimeout> | null = null;

  wheel = (e: WheelEvent): boolean => {
    if (ui.editorTab !== 'masks' || !held('resize', e)) return false;
    const layer = editor.activeLayer();
    const comp = editor.activeMaskComponent();
    if (!layer || !comp || !comp.enabled) return false;
    const meta = editor.meta;
    const source = meta ? { w: meta.source_w, h: Math.max(meta.source_h, 1) } : null;
    const axis = held('feather', e) ? 'feather' : 'size';
    const next = wheeledKind(comp.kind, axis, wheelNotches(e), source ? source.w / source.h : 1);
    if (!next) return false;
    e.preventDefault();
    e.stopPropagation();
    editor.updateMaskComponentKind(layer.id, comp.id, next, true);
    this.#show(
      axis === 'feather'
        ? { label: 'Feather', value: shapeFeatherLabel(next), keys: 'maskShapeFeatherWheel' }
        : { label: 'Size', value: shapeExtentLabel(next, source), keys: 'maskShapeSizeWheel' }
    );
    this.#queueCommit();
    return true;
  };

  #show({
    label,
    value,
    keys
  }: {
    label: string;
    value: string;
    keys: 'maskShapeSizeWheel' | 'maskShapeFeatherWheel';
  }): void {
    this.pill = { lead: label, label: value, tail: '', mono: true, hint: keysFor(keys) };
    if (this.#flash) clearTimeout(this.#flash);
    this.#flash = setTimeout(() => {
      this.pill = null;
      this.#flash = null;
    }, FLASH_MS);
  }

  #queueCommit(): void {
    if (this.#settle) clearTimeout(this.#settle);
    this.#settle = setTimeout(() => {
      this.#settle = null;
      void editor.commitMasks();
    }, SETTLE_MS);
  }
}

export const shapeWheel = new ShapeWheel();
