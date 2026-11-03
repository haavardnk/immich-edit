import { CONTEXT_STACKS } from '#lib/shortcuts/context.ts';
import { KEYBINDS, type Keybind, type KeybindContext } from '#lib/shortcuts/table.ts';
import { describe, expect, it, vi } from 'vitest';
import { commandLabel } from './command';
import { COMMANDS } from './registry';

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/state', () => ({ page: { url: new URL('http://localhost/'), params: {} } }));

const AWAITING_CONTEXTS: readonly KeybindContext[] = ['grid', 'loupe', 'compare', 'survey'];

const OVERLAY_BINDS = ['maskCancelDraw', 'maskClosePolygon', 'maskDelete', 'maskNudge'];

const BINDS = KEYBINDS as readonly Keybind[];

function stackOf(context: KeybindContext): readonly KeybindContext[] {
  return (
    (CONTEXT_STACKS as Partial<Record<KeybindContext, readonly KeybindContext[]>>)[context] ?? [
      context
    ]
  );
}

describe('command registry', () => {
  it('has unique ids and a label for every command', () => {
    const ids = COMMANDS.map((command) => command.id);
    expect(new Set(ids).size).toBe(ids.length);
    for (const command of COMMANDS) expect(commandLabel(command), command.id).not.toBe(command.id);
  });

  it('binds only keyed shortcuts, in contexts the shortcut lists', () => {
    for (const command of COMMANDS.filter((entry) => entry.bind)) {
      const bind = BINDS.find(({ id }) => id === command.bind);
      expect(bind?.keys.length, command.id).toBeGreaterThan(0);
      for (const context of command.contexts) expect(bind?.contexts, command.id).toContain(context);
    }
  });

  it('runs every keyed shortcut through a command', () => {
    const missing = BINDS.filter(
      (bind) => bind.keys.length > 0 && !OVERLAY_BINDS.includes(bind.id)
    ).flatMap((bind) =>
      bind.contexts
        .filter((context) => !AWAITING_CONTEXTS.includes(context))
        .filter(
          (context) =>
            !COMMANDS.some(
              (command) =>
                command.bind === bind.id &&
                command.contexts.some((owner) => stackOf(context).includes(owner))
            )
        )
        .map((context) => `${bind.id}@${context}`)
    );
    expect(missing).toEqual([]);
  });

  it('dismisses the innermost layer first on Escape', () => {
    expect(
      COMMANDS.filter((command) => command.bind === 'editorEscape').map(({ id }) => id)
    ).toEqual([
      'editor.closePopovers',
      'editor.closeHelp',
      'geometry.closeTool',
      'retouch.stopSampling',
      'retouch.deselect',
      'masks.deselectShape',
      'editor.exitFullscreen'
    ]);
  });

  it('leaves only the last command on a shared shortcut without an active guard', () => {
    const ambiguous = COMMANDS.filter((command, index) =>
      COMMANDS.slice(index + 1).some(
        (later) =>
          later.bind === command.bind &&
          later.contexts.some((context) => command.contexts.includes(context))
      )
    ).filter((command) => !command.active);
    expect(ambiguous.map(({ id }) => id)).toEqual([]);
  });
});
