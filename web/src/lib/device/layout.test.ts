import { afterEach, describe, expect, it, vi } from 'vitest';

type Listener = (e: KeyboardEvent) => void;

function stubWindow() {
  const win = { addEventListener: vi.fn(), removeEventListener: vi.fn() };
  vi.stubGlobal('window', win);
  return win;
}

function stubStorage(initial?: unknown): void {
  const values = new Map<string, string>();
  if (initial !== undefined) values.set('immich-edit:layout', JSON.stringify(initial));
  vi.stubGlobal('localStorage', {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value)
  });
}

function keydown(tagName: string): KeyboardEvent {
  return { target: { tagName, isContentEditable: false } } as unknown as KeyboardEvent;
}

async function install() {
  stubStorage();
  const win = stubWindow();
  const { installLayout, layout } = await import('#lib/device/layout.svelte.ts');
  const cleanup = installLayout();
  const listener = win.addEventListener.mock.calls[0]?.[1] as Listener;
  return { win, layout, cleanup, listener };
}

afterEach(() => {
  vi.unstubAllGlobals();
  vi.resetModules();
});

describe('hardware keyboard detection', () => {
  it('ignores keys typed into an editable element', async () => {
    const { win, layout, listener } = await install();

    listener(keydown('INPUT'));
    listener(keydown('TEXTAREA'));

    expect(layout.hasKeyboard).toBe(false);
    expect(win.removeEventListener).not.toHaveBeenCalled();
  });

  it('notes a hardware key and stops listening', async () => {
    const { win, layout, listener } = await install();

    listener(keydown('DIV'));

    expect(layout.hasKeyboard).toBe(true);
    expect(win.removeEventListener).toHaveBeenCalledWith('keydown', listener, { capture: true });
  });

  it('removes the listener on cleanup', async () => {
    const { win, cleanup, listener } = await install();

    cleanup();

    expect(win.addEventListener).toHaveBeenCalledWith('keydown', listener, { capture: true });
    expect(win.removeEventListener).toHaveBeenCalledWith('keydown', listener, { capture: true });
  });
});

describe('stored layout choice', () => {
  it.each([
    ['a valid choice', { choice: 'touch' }, 'touch'],
    ['an unknown choice', { choice: 'tablet' }, 'auto'],
    ['nothing', undefined, 'auto']
  ])('restores %s', async (_name, stored, expected) => {
    stubStorage(stored);
    const { layout } = await import('#lib/device/layout.svelte.ts');

    expect(layout.choice).toBe(expected);
  });
});
