import type { KeybindContext } from '#lib/shortcuts/table.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createKeyDispatcher } from './keys';
import type { Command } from './types';

const h = vi.hoisted(() => ({
  modalOpen: false,
  helpOpen: false,
  toast: vi.fn()
}));

vi.mock('@immich/ui', () => ({ isModalOpen: () => h.modalOpen }));
vi.mock('#lib/stores/toasts.svelte.ts', () => ({ toasts: { push: h.toast } }));
vi.mock('#lib/stores/ui.svelte.ts', () => ({
  ui: {
    get keybindsHelpOpen() {
      return h.helpOpen;
    }
  }
}));

const EDITOR: readonly KeybindContext[] = ['editor', 'global'];
const MASKS: readonly KeybindContext[] = ['masks', 'editor', 'global'];

function press(key: string, init: { code?: string; target?: object } = {}): KeyboardEvent {
  const event = {
    key,
    code: init.code ?? '',
    metaKey: false,
    ctrlKey: false,
    shiftKey: false,
    altKey: false,
    target: init.target ?? null,
    defaultPrevented: false,
    getModifierState: () => false,
    preventDefault: vi.fn(() => {
      event.defaultPrevented = true;
    })
  };
  return event as unknown as KeyboardEvent;
}

function command(overrides: Partial<Command> & Pick<Command, 'id'>): Command {
  return { contexts: ['editor'], run: vi.fn(), ...overrides };
}

const input = { tagName: 'INPUT', isContentEditable: false };

const radio = {
  tagName: 'BUTTON',
  isContentEditable: false,
  closest: (selector: string) => (selector === '[role="radiogroup"]' ? {} : null),
  matches: () => true
};

const button = {
  tagName: 'BUTTON',
  isContentEditable: false,
  closest: () => null,
  matches: () => true
};

describe('key dispatcher', () => {
  beforeEach(() => {
    h.modalOpen = false;
    h.helpOpen = false;
    h.toast.mockReset();
  });

  it('runs the first active command for the shortcut and consumes the key', () => {
    const inactive = command({ id: 'a', bind: 'favorite', active: () => false });
    const active = command({ id: 'b', bind: 'favorite' });
    const event = press('p');

    createKeyDispatcher([inactive, active]).keydown(event, EDITOR);

    expect(inactive.run).not.toHaveBeenCalled();
    expect(active.run).toHaveBeenCalledWith({ key: 'p' });
    expect(event.preventDefault).toHaveBeenCalled();
  });

  it('ignores commands owned by a context outside the stack', () => {
    const masks = command({ id: 'masked', contexts: ['masks'], bind: 'favorite' });
    const keys = createKeyDispatcher([masks]);
    const outside = press('p');

    keys.keydown(outside, EDITOR);
    expect(masks.run).not.toHaveBeenCalled();
    expect(outside.preventDefault).not.toHaveBeenCalled();
    keys.keydown(press('p'), MASKS);
    expect(masks.run).toHaveBeenCalledOnce();
  });

  it('consumes a disabled command without running it and explains why', () => {
    const paste = command({
      id: 'paste',
      bind: 'pasteEdits',
      enabled: () => false,
      unavailable: () => 'Nothing copied'
    });
    const event = press('v');
    Object.assign(event, { metaKey: true, ctrlKey: true });

    createKeyDispatcher([paste]).keydown(event, EDITOR);

    expect(paste.run).not.toHaveBeenCalled();
    expect(event.preventDefault).toHaveBeenCalled();
    expect(h.toast).toHaveBeenCalledWith('info', 'Nothing copied', 3000);
  });

  it('leaves a yielded key to the browser', () => {
    const nav = command({ id: 'nav', bind: 'editorNav', yields: () => true });
    const event = press('ArrowLeft');

    createKeyDispatcher([nav]).keydown(event, EDITOR);

    expect(nav.run).not.toHaveBeenCalled();
    expect(event.preventDefault).not.toHaveBeenCalled();
  });

  it.each([
    ['default', false, false],
    ['outsideFields', false, true],
    ['always', true, true]
  ] as const)('applies the %s gate to fields and the help dialog', (gate, inField, withHelp) => {
    const bound = command({ id: 'zoom', bind: 'zoomToggle', gate });
    const keys = createKeyDispatcher([bound]);

    keys.keydown(press('z', { target: input }), EDITOR);
    expect(bound.run).toHaveBeenCalledTimes(inField ? 1 : 0);

    h.helpOpen = true;
    keys.keydown(press('z'), EDITOR);
    expect(bound.run).toHaveBeenCalledTimes((inField ? 1 : 0) + (withHelp ? 1 : 0));
  });

  it.each([
    ['an already handled key', press('z'), () => true],
    ['a keyboard-focused radio group', press('z', { target: radio }), () => false],
    ['Enter on a focused control', press('Enter', { target: button }), () => false]
  ])('keeps default-gated commands away from %s', (_, event, handled) => {
    const bound = command({ id: 'zoom', bind: 'zoomToggle' });
    const open = command({ id: 'open', bind: 'openEditor', contexts: ['grid'] });
    Object.assign(event, { defaultPrevented: handled() });

    createKeyDispatcher([bound, open]).keydown(event, [...EDITOR, 'grid']);

    expect(bound.run).not.toHaveBeenCalled();
    expect(open.run).not.toHaveBeenCalled();
  });

  it('does nothing while a modal is open', () => {
    const bound = command({ id: 'zoom', bind: 'zoomToggle', gate: 'always' });
    h.modalOpen = true;

    createKeyDispatcher([bound]).keydown(press('z'), EDITOR);

    expect(bound.run).not.toHaveBeenCalled();
  });

  it('releases a held command on key up by code or by its shortcut', () => {
    const release = vi.fn();
    const hold = command({ id: 'hold', bind: 'holdOriginal', release });
    const keys = createKeyDispatcher([hold]);

    keys.keydown(press('\\', { code: 'Backslash' }), EDITOR);
    keys.keyup(press('|', { code: 'Backslash' }));
    expect(release).toHaveBeenCalledOnce();

    keys.keyup(press('<', { code: 'IntlBackslash' }));
    expect(release).toHaveBeenCalledTimes(2);

    keys.keyup(press('x', { code: 'KeyX' }));
    expect(release).toHaveBeenCalledTimes(2);
  });
});
