import { gridHost, type GridHost } from '#lib/commands/grid/gridHost.ts';
import type { Command, CommandTrigger } from '#lib/commands/types.ts';
import type { KeybindContext, KeybindId } from '#lib/shortcuts/table.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';

export interface GridCommandSpec {
  id: string;
  bind: KeybindId;
  contexts?: readonly KeybindContext[];
  shown?: () => boolean;
  unavailable?: () => string | null;
  yields?: (host: GridHost, trigger: CommandTrigger) => boolean;
  run: (host: GridHost, trigger?: CommandTrigger) => void;
}

export function gridCommand(spec: GridCommandSpec): Command {
  return {
    id: spec.id,
    contexts: spec.contexts ?? ['grid'],
    bind: spec.bind,
    gate: 'outsideControls',
    active: () => browseView.loupeId === null && gridHost.get() !== null,
    enabled: () => !gridHost.get()?.selectingAll(),
    shown: spec.shown,
    unavailable: spec.unavailable,
    yields: (trigger) => {
      const host = gridHost.get();
      if (!host || host.selectingAll()) return false;
      return spec.yields?.(host, trigger) ?? false;
    },
    run: (trigger) => {
      const host = gridHost.get();
      if (host) spec.run(host, trigger);
    }
  };
}
