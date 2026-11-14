import { layout } from '#lib/device/layout.svelte.ts';
import type { PillText } from '#lib/editor/pill.ts';
import { keyLabel } from '#lib/shortcuts/chord.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { countOf } from '#lib/utils/plural.ts';

export function rangeSamplePill(): PillText | null {
  const picker = editor.rangePicker;
  if (!picker?.ready) return null;
  if (layout.mode === 'touch') {
    return { lead: '', label: 'Lift to sample', tail: '', mono: false };
  }
  const count = picker.samples.length;
  return {
    lead: '',
    label: count > 0 ? countOf(count, 'sample') : 'Sample',
    tail: count > 0 ? '' : '· click or drag on the photo',
    mono: false,
    hint: `${keyLabel('Shift')}-click adds · ${keyLabel('Escape')} finishes`
  };
}
