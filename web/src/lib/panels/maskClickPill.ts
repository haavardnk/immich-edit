import { layout } from '#lib/device/layout.svelte.ts';
import type { PillText } from '#lib/editor/pill.ts';
import { keyLabel } from '#lib/shortcuts/chord.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { countOf } from '#lib/utils/plural.ts';

export function clickPointCount(): number {
  const comp = editor.activeMaskComponent();
  return comp?.generated?.kind === 'click' ? (comp.generated.points?.length ?? 0) : 0;
}

export function maskClickPill(): PillText | null {
  const tool = editor.clickTool;
  if (!tool.active) return null;
  const touch = layout.mode === 'touch';
  if (tool.box) {
    return {
      lead: '',
      label: 'Box select',
      tail: '· drag around the subject',
      mono: false,
      ...(touch ? {} : { hint: `${keyLabel('Escape')} cancels` })
    };
  }
  const points = clickPointCount();
  return {
    lead: '',
    label: 'Click to select',
    tail: points > 0 ? `· ${countOf(points, 'point')}` : '',
    mono: false,
    hint: touch ? 'Done finishes' : `${keyLabel('Escape')} or Done finishes`
  };
}
