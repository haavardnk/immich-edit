import type { PillText } from '#lib/editor/pill.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { hoverPreview, maskHover } from '#lib/stores/editor/maskHover.svelte.ts';
import { modeVerb } from './maskTools';

export function maskHoverPill(): PillText | null {
  const hover = maskHover.target;
  if (!hover || !hoverPreview(editor, hover)) return null;
  return {
    lead: 'Previewing',
    label: hover.label,
    tail: hover.mode ? `· ${modeVerb(hover.mode)}` : '',
    mono: false
  };
}
