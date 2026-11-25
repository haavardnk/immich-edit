import { loupeFocusedId } from '#lib/browse/loupeActions.ts';
import type { Command } from '#lib/commands/types.ts';
import type { KeybindContext } from '#lib/shortcuts/table.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';

export function withFocused(run: (id: string) => void): () => void {
  return () => {
    const id = loupeFocusedId();
    if (id) run(id);
  };
}

export const VIEWING: readonly KeybindContext[] = ['loupe', 'compare', 'survey'];

export function loupeCommand<T extends Command>(spec: T): T {
  return {
    ...spec,
    gate: spec.gate ?? 'outsideWidgets',
    active: (trigger) =>
      browseView.loupeId !== null &&
      !browseView.loupeBehindEditor &&
      (spec.active?.(trigger) ?? true)
  };
}
