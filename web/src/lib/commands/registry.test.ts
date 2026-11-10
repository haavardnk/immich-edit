import { CONTEXT_STACKS } from '#lib/shortcuts/context.ts';
import { compare, COMPARE_PANES } from '#lib/stores/compare.svelte.ts';
import { KEYBINDS, type Keybind, type KeybindContext } from '#lib/shortcuts/table.ts';
import { describe, expect, it, vi } from 'vitest';
import { commandLabel, commandShortLabel, commandTitle, isEnabled, isShown } from './command';
import * as menus from './menus';
import { COMMANDS } from './registry';
import type { Command } from './types';

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/state', () => ({ page: { url: new URL('http://localhost/'), params: {} } }));

const OVERLAY_BINDS = [
  'maskCancelDraw',
  'maskClosePolygon',
  'maskDelete',
  'maskNudge',
  'pinSetting'
];

const SHARED_BINDS = [
  'editorEscape@editor',
  'resetEdits@editor',
  'loupeClose@loupe',
  'compareExit@compare',
  'surveyExit@survey',
  'gridClearSelection@grid'
];

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

  it('names menus in full and compact controls by a shorter label', () => {
    const single = COMMANDS.find((command) => command.id === 'loupe.single');
    expect(single && commandLabel(single)).toBe('Single photo');
    expect(single && commandShortLabel(single)).toBe('Single');
    for (const command of COMMANDS.filter((entry) => entry.short)) {
      expect(commandShortLabel(command).length, command.id).toBeLessThan(
        commandLabel(command).length
      );
    }
  });

  it('hides survey and says why on a layout that holds two panes', () => {
    const surveys = COMMANDS.filter(({ id }) => id === 'loupe.survey' || id === 'grid.survey');
    expect(surveys).toHaveLength(2);
    const release = compare.limitBy(() => COMPARE_PANES);
    try {
      for (const command of surveys) {
        expect(isShown(command), command.id).toBe(false);
        expect(isEnabled(command), command.id).toBe(false);
        expect(commandTitle(command), command.id).toBe('Survey needs a larger screen');
      }
    } finally {
      release();
    }
    for (const command of surveys) expect(isEnabled(command), command.id).toBe(true);
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
      'develop.exitTarget',
      'geometry.closeTool',
      'retouch.stopSampling',
      'retouch.deselect',
      'masks.deselectShape',
      'editor.exitFullscreen'
    ]);
  });

  it.each([
    ['loupeClose', ['loupe.exitFullscreen', 'loupe.close']],
    ['compareExit', ['compare.exitFullscreen', 'compare.exit']],
    ['surveyExit', ['survey.exitFullscreen', 'survey.exit']]
  ])('leaves loupe fullscreen before %s', (bind, order) => {
    expect(COMMANDS.filter((command) => command.bind === bind).map(({ id }) => id)).toEqual(order);
  });

  it('shares a shortcut within a context only in an allowlisted guarded chain', () => {
    const owners = new Map<string, Command[]>();
    for (const command of COMMANDS) {
      for (const context of command.contexts) {
        const key = `${command.bind}@${context}`;
        if (command.bind) owners.set(key, [...(owners.get(key) ?? []), command]);
      }
    }
    const shared = [...owners].filter(([, commands]) => commands.length > 1);
    expect(shared.map(([key]) => key).sort()).toEqual([...SHARED_BINDS].sort());
    for (const [key, commands] of shared) {
      const unguarded = commands.slice(0, -1).filter((command) => !command.active);
      expect(
        unguarded.map(({ id }) => id),
        key
      ).toEqual([]);
    }
  });

  it('lists registered commands in every menu', () => {
    const ids = Object.fromEntries(
      Object.entries(menus).map(([name, menu]) => [
        name,
        menu.map((group) => group.map((command) => command.id))
      ])
    );
    expect(ids).toEqual({
      LOUPE_VIEW_MODE_MENU: [['loupe.single', 'loupe.compare', 'loupe.survey']],
      LOUPE_MORE_MENU: [
        ['loupe.fullscreen', 'loupe.clipWarn', 'app.greyCanvas'],
        ['loupe.trash'],
        ['app.help']
      ],
      LOUPE_PHONE_MORE_MENU: [
        ['loupe.autoAdvance'],
        ['loupe.zoom', 'loupe.filmstrip', 'loupe.fullscreen', 'loupe.clipWarn', 'app.greyCanvas'],
        ['loupe.compare', 'loupe.survey'],
        ['loupe.trash']
      ],
      LOUPE_PHONE_MULTI_MORE_MENU: [
        ['loupe.autoAdvance'],
        [
          'loupe.info',
          'loupe.zoom',
          'loupe.filmstrip',
          'loupe.fullscreen',
          'loupe.clipWarn',
          'app.greyCanvas'
        ],
        ['loupe.trash']
      ],
      LOUPE_WIDE_MORE_MENU: [
        ['loupe.autoAdvance'],
        ['loupe.fullscreen', 'loupe.clipWarn', 'app.greyCanvas'],
        ['loupe.trash']
      ],
      EDITOR_VERSION_MENU: [['editor.virtualCopy']],
      EDITOR_VIEW_MENU: [
        [
          'editor.info',
          'editor.fullscreen',
          'editor.holdOriginal',
          'editor.beforeAfter',
          'editor.clipWarn',
          'app.greyCanvas'
        ]
      ],
      EDITOR_MORE_MENU: [['editor.trash'], ['app.help']],
      EDITOR_PALETTE_MENU: [
        [
          'editor.auto',
          'editor.resetDevelop',
          'editor.copy',
          'editor.paste',
          'editor.undo',
          'editor.redo',
          'editor.virtualCopy'
        ],
        [
          'editor.beforeAfter',
          'editor.clipWarn',
          'editor.info',
          'editor.fullscreen',
          'app.greyCanvas',
          'app.help'
        ]
      ]
    });
    for (const command of Object.values(menus).flatMap((menu) => menu.flat())) {
      expect(COMMANDS, command.id).toContain(command);
    }
  });

  it('creates one editor virtual copy per key press', () => {
    const command = COMMANDS.find(({ id }) => id === 'editor.virtualCopy');
    const trigger = { key: "'", code: 'Quote', shiftKey: false, repeat: false };
    expect(command?.yields?.({ ...trigger, repeat: true })).toBe(true);
    expect(command?.yields?.(trigger)).toBe(false);
  });
});
