import { beforeEach, describe, expect, it, vi, type MockInstance } from 'vitest';
import { DEFAULT_COPY_SECTIONS } from '#lib/editor/copyPaste.ts';
import { keysFor } from '#lib/shortcuts/labels.ts';
import { clipboard } from '#lib/stores/clipboard.svelte.ts';
import { copyDialog } from '#lib/stores/copyDialog.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { neutralEdits, type Edits } from '#lib/types/edits.ts';
import {
  canPaste,
  canResetDevelop,
  copyEdits,
  pasteEdits,
  resetDevelop,
  type DevelopCtx
} from './develop.svelte';

function edited(): Edits {
  const edits = neutralEdits();
  edits.basic.exposure_ev = 1;
  return edits;
}

function context(edits: Edits): DevelopCtx {
  return {
    assetId: 'asset-1',
    initialised: true,
    edits,
    saving: false,
    autoBusy: false,
    error: null,
    onLive: vi.fn(),
    onCommit: vi.fn(() => Promise.resolve())
  };
}

describe('develop feedback', () => {
  let push: MockInstance<typeof toasts.push>;

  beforeEach(() => {
    vi.restoreAllMocks();
    clipboard.clear();
    copyDialog.cancel();
    push = vi.spyOn(toasts, 'push').mockImplementation(() => undefined);
  });

  it.each([
    [
      'copy without edits',
      () => copyEdits(context(neutralEdits())),
      'This photo has no edits to copy'
    ],
    ['paste with nothing copied', () => pasteEdits(context(edited())), 'Nothing copied'],
    [
      'reset',
      () => resetDevelop(context(edited())),
      `Reset develop settings. Undo with ${keysFor('undo')}.`
    ]
  ])('%s toasts', async (_name, run, message) => {
    await run();
    expect(push).toHaveBeenCalledWith(expect.any(String), message, { ttlMs: 3000 });
  });

  it('confirms a copy only after the dialog copies', () => {
    copyEdits(context(edited()));
    expect(push).not.toHaveBeenCalled();

    copyDialog.confirm();

    expect(push).toHaveBeenCalledWith('success', 'Copied settings', { ttlMs: 3000 });
  });

  it('pastes as one history step and offers undo', async () => {
    clipboard.copy(edited(), { ...DEFAULT_COPY_SECTIONS });
    const ctx = context(neutralEdits());

    await pasteEdits(ctx);

    expect(ctx.edits.basic.exposure_ev).toBe(1);
    expect(ctx.onCommit).toHaveBeenCalledWith('Paste');
    expect(push).toHaveBeenCalledWith('success', `Pasted settings. Undo with ${keysFor('undo')}.`, {
      ttlMs: 3000
    });
  });
});

describe('develop guards', () => {
  let push: MockInstance<typeof toasts.push>;

  beforeEach(() => {
    vi.restoreAllMocks();
    clipboard.clear();
    push = vi.spyOn(toasts, 'push').mockImplementation(() => undefined);
  });

  function saving(edits: Edits): DevelopCtx {
    return { ...context(edits), saving: true };
  }

  it.each([
    ['an unedited photo', () => context(neutralEdits())],
    ['a save in flight', () => saving(edited())],
    ['no photo', () => ({ ...context(edited()), assetId: null })]
  ])('does not reset %s', async (_name, make) => {
    const ctx = make();
    const before = ctx.edits;

    expect(canResetDevelop(ctx)).toBe(false);
    await resetDevelop(ctx);

    expect(ctx.edits).toBe(before);
    expect(ctx.onCommit).not.toHaveBeenCalled();
    expect(push).not.toHaveBeenCalled();
  });

  it('does not paste while a save is in flight', async () => {
    clipboard.copy(edited(), { ...DEFAULT_COPY_SECTIONS });
    const ctx = saving(neutralEdits());

    expect(canPaste(ctx)).toBe(false);
    await pasteEdits(ctx);

    expect(ctx.edits.basic.exposure_ev).toBe(0);
    expect(ctx.onCommit).not.toHaveBeenCalled();
    expect(push).not.toHaveBeenCalled();
  });

  it('allows paste only with something copied', () => {
    const ctx = context(neutralEdits());
    expect(canPaste(ctx)).toBe(false);

    clipboard.copy(edited(), { ...DEFAULT_COPY_SECTIONS });

    expect(canPaste(ctx)).toBe(true);
    expect(canResetDevelop(context(edited()))).toBe(true);
  });
});
