import { isKeybind, matchKeybind } from '#lib/shortcuts/match.ts';
import type { KeybindContext } from '#lib/shortcuts/table.ts';
import { isRadioGroupTarget, isTypingTarget, yieldsToControl } from '#lib/shortcuts/targets.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import { escapeTaken, isAnyModalOpen } from '#lib/stores/overlays.ts';
import { isEnabled } from '#lib/commands/command.ts';
import type { Command, CommandTrigger, KeyGate } from '#lib/commands/types.ts';

export interface KeyDispatcher {
  keydown: (e: KeyboardEvent, contexts: readonly KeybindContext[]) => void;
  keyup: (e: KeyboardEvent) => void;
}

function passes(gate: KeyGate, e: KeyboardEvent): boolean {
  if (gate === 'always') return true;
  if (isTypingTarget(e)) return false;
  if (gate === 'outsideFields') return true;
  if (yieldsToControl(e)) return false;
  if (gate === 'outsideControls') return true;
  if (isRadioGroupTarget(e)) return false;
  if (gate === 'outsideWidgets') return true;
  return !e.defaultPrevented && !ui.keybindsHelpOpen;
}

export function createKeyDispatcher(commands: readonly Command[]): KeyDispatcher {
  const held = new Map<Command, string>();

  function resolve(
    e: KeyboardEvent,
    contexts: readonly KeybindContext[],
    trigger: CommandTrigger
  ): Command | null {
    const bind = matchKeybind(e, contexts);
    if (!bind) return null;
    const owned = commands.filter(
      (command) =>
        command.bind === bind && command.contexts.some((context) => contexts.includes(context))
    );
    return owned.find((command) => command.active?.(trigger) ?? true) ?? null;
  }

  return {
    keydown: (e, contexts) => {
      if (isAnyModalOpen() || escapeTaken(e)) return;
      const trigger = { key: e.key, code: e.code, shiftKey: e.shiftKey, repeat: e.repeat };
      const command = resolve(e, contexts, trigger);
      if (!command || !passes(command.gate ?? 'default', e) || command.yields?.(trigger)) return;
      e.preventDefault();
      if (!isEnabled(command)) {
        const reason = command.unavailable?.();
        if (reason) toasts.push('info', reason, { ttlMs: 3000 });
        return;
      }
      if (command.release) held.set(command, e.code);
      command.run(trigger);
    },
    keyup: (e) => {
      for (const command of commands) {
        if (!command.release) continue;
        const bound = !!command.bind && isKeybind(e, command.bind);
        if (held.get(command) !== e.code && !bound) continue;
        held.delete(command);
        command.release();
      }
    }
  };
}
