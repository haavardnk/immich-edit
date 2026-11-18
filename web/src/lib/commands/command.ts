import { hint } from '#lib/shortcuts/labels.ts';
import { KEYBINDS, type Keybind } from '#lib/shortcuts/table.ts';
import type { Command, CommandTrigger } from '#lib/commands/types.ts';

export function commandLabel(command: Command): string {
  const bind = (KEYBINDS as readonly Keybind[]).find(({ id }) => id === command.bind);
  const label = typeof command.label === 'function' ? command.label() : command.label;
  return label ?? bind?.label ?? command.id;
}

export function isEnabled(command: Command): boolean {
  return command.enabled?.() ?? true;
}

export function commandTitle(command: Command): string {
  const reason = isEnabled(command) ? null : command.unavailable?.();
  if (reason) return reason;
  const label = commandLabel(command);
  return command.bind ? hint(label, command.bind) : label;
}

export function runCommand(command: Command, trigger?: CommandTrigger): void {
  if (isEnabled(command)) command.run(trigger);
}

export interface PressHandlers {
  onclick?: () => void;
  onpointerdown?: () => void;
  onpointerup?: () => void;
  onpointercancel?: () => void;
  onpointerleave?: () => void;
}

export function pressHandlers(command: Command, before?: () => void): PressHandlers {
  const release = command.release;
  if (!release) {
    return {
      onclick: () => {
        before?.();
        runCommand(command);
      }
    };
  }
  return {
    onpointerdown: () => runCommand(command),
    onpointerup: release,
    onpointercancel: release,
    onpointerleave: release
  };
}
