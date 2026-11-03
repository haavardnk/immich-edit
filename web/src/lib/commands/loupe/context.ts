import type { Command } from '#lib/commands/types.ts';
import type { KeybindContext } from '#lib/shortcuts/table.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';

export const VIEWING: readonly KeybindContext[] = ['loupe', 'compare', 'survey'];

export function loupeCommand(spec: Command): Command {
  return {
    ...spec,
    gate: spec.gate ?? 'outsideWidgets',
    active: (trigger) => browseView.loupeId !== null && (spec.active?.(trigger) ?? true)
  };
}
