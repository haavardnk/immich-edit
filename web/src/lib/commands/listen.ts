import { commandKeys } from '#lib/commands/registry.ts';
import { activeContexts } from '#lib/shortcuts/context.ts';

function onKeydown(e: KeyboardEvent): void {
  commandKeys.keydown(e, activeContexts());
}

export function listenForCommands(): () => void {
  window.addEventListener('keydown', onKeydown);
  window.addEventListener('keyup', commandKeys.keyup);
  return () => {
    window.removeEventListener('keydown', onKeydown);
    window.removeEventListener('keyup', commandKeys.keyup);
  };
}
