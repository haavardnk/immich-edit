import type { EditHistoryEntry } from '#lib/api/edits.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import {
  layout,
  layoutControls,
  renderLayout,
  settle,
  type ControlRef
} from '#lib/testing/layoutParity.ts';
import { neutralEdits, type Edits } from '#lib/types/edits.ts';
import { flushSync } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import HistoryList from './HistoryList.svelte';
import HistoryPopover from './HistoryPopover.svelte';

const api = vi.hoisted(() => ({ listEditHistory: vi.fn() }));

vi.mock('#lib/api/edits.ts', async (importOriginal) => ({
  ...(await importOriginal<typeof import('#lib/api/edits.ts')>()),
  listEditHistory: api.listEditHistory
}));

const button = (name: string): ControlRef => ({ role: 'button', name });

const POPOVER = '[data-popover-content]';

function exposure(value: number): Edits {
  const edits = neutralEdits();
  edits.basic.exposure_ev = value;
  return edits;
}

function entry(id: number, undone = false): EditHistoryEntry {
  return {
    id,
    manifest_hash: `h${id}`,
    deleted: false,
    edits: null,
    created_at: '2026-01-01T10:00:00Z',
    action: null,
    undone
  };
}

function withEdits(id: number, undone = false): EditHistoryEntry {
  return { ...entry(id, undone), edits: exposure(id) };
}

function step(id: number): HTMLElement {
  const row = document.body.querySelector<HTMLElement>(`li[data-history-step="${id}"]`);
  if (!row) throw new Error(`Missing step ${id}`);
  return row;
}

function jump(id: number): HTMLButtonElement {
  const target = step(id).querySelector<HTMLButtonElement>('button:not([aria-controls])');
  if (!target) throw new Error(`Missing jump for ${id}`);
  return target;
}

function click(selector: string): void {
  const target = document.body.querySelector<HTMLButtonElement>(selector);
  if (!target) throw new Error(`Missing ${selector}`);
  target.click();
  flushSync();
}

function pointer(target: Element, type: string, pointerType = 'mouse'): void {
  target.dispatchEvent(new PointerEvent(type, { bubbles: type !== 'pointerenter', pointerType }));
  flushSync();
}

async function closed(): Promise<void> {
  await vi.waitFor(() => expect(document.body.querySelector(POPOVER)).toBeNull());
}

function openPopover(): void {
  click('button[aria-label="Edit history"]');
}

beforeEach(() => {
  api.listEditHistory.mockResolvedValue([entry(2), entry(1)]);
  editor.assetId = 'a1';
  toasts.items = [];
});

afterEach(() => {
  editor.historySteps.leave();
  editor.historySteps.reset();
  editor.assetId = null;
  api.listEditHistory.mockReset();
  vi.restoreAllMocks();
});

describe('HistoryList layout parity', () => {
  it('exposes the same controls standalone and in the history popover', async () => {
    const found = await layoutControls([
      layout('body', HistoryList, {}),
      layout(
        'popover',
        HistoryPopover,
        {},
        {
          open: openPopover,
          exclude: [button('Edit history'), button('Close'), button('New snapshot')]
        }
      )
    ]);
    expect(found.body.length).toBeGreaterThanOrEqual(4);
    expect(found.popover).toEqual(found.body);
  });
});

describe('HistoryPopover', () => {
  it('loads the history once per saved state and restore', async () => {
    const destroy = renderLayout(layout('popover', HistoryPopover, {}));
    await settle();
    expect(api.listEditHistory).not.toHaveBeenCalled();

    openPopover();
    await settle();
    expect(api.listEditHistory).toHaveBeenCalledTimes(1);

    click(`${POPOVER} button[aria-label="Close"]`);
    await closed();

    openPopover();
    await settle();
    expect(api.listEditHistory).toHaveBeenCalledTimes(1);

    editor.savedHash = 'h3';
    await settle();
    expect(api.listEditHistory).toHaveBeenCalledTimes(2);
    editor.restoreCount++;
    await settle();
    expect(api.listEditHistory).toHaveBeenCalledTimes(3);
    editor.savedHash = '';
    await destroy();
  });

  it('stays open after a restore', async () => {
    api.listEditHistory.mockResolvedValue([withEdits(2), withEdits(1)]);
    const restore = vi.spyOn(editor, 'restoreHistoryEntry').mockResolvedValue(true);
    const destroy = renderLayout(layout('popover', HistoryPopover, {}));
    openPopover();
    await settle();

    jump(1).click();
    await settle();
    expect(restore).toHaveBeenCalledWith(
      expect.objectContaining({ id: 1, edits: exposure(1), from: 2 })
    );
    expect(document.body.querySelector(POPOVER)).not.toBeNull();
    await destroy();
  });
});

describe('HistoryList', () => {
  it('marks the newest entry that is not undone as current', async () => {
    api.listEditHistory.mockResolvedValue([entry(3, true), entry(2), entry(1)]);
    const destroy = renderLayout(layout('body', HistoryList, {}));
    await settle();

    expect(jump(2).getAttribute('aria-current')).toBe('step');
    expect(jump(3).getAttribute('aria-current')).toBeNull();
    expect(step(3).hasAttribute('data-undone')).toBe(true);
    expect(jump(3).getAttribute('aria-label')).toBe('Jump forward to h3');
    await destroy();
  });

  it('restores an undone entry forward from the current one', async () => {
    api.listEditHistory.mockResolvedValue([withEdits(3, true), withEdits(2), withEdits(1)]);
    const restore = vi.spyOn(editor, 'restoreHistoryEntry').mockImplementation(() => {
      editor.restoreCount++;
      return Promise.resolve(true);
    });
    const destroy = renderLayout(layout('body', HistoryList, {}));
    await settle();

    jump(3).click();
    await settle();
    expect(restore).toHaveBeenCalledWith(expect.objectContaining({ id: 3, from: 2 }));
    expect(api.listEditHistory).toHaveBeenCalledTimes(2);
    await destroy();
  });

  it('ignores a click on the current entry', async () => {
    const restore = vi.spyOn(editor, 'restoreHistoryEntry').mockResolvedValue(true);
    const destroy = renderLayout(layout('body', HistoryList, {}));
    await settle();

    jump(2).click();
    await settle();
    expect(restore).not.toHaveBeenCalled();
    await destroy();
  });

  it('previews a hovered mouse step and fades the newer ones', async () => {
    const older = { ...entry(1), edits: exposure(1) };
    api.listEditHistory.mockResolvedValue([{ ...entry(2), edits: exposure(2) }, older]);
    const destroy = renderLayout(layout('body', HistoryList, {}));
    await settle();

    pointer(jump(1), 'pointerenter', 'touch');
    expect(editor.editsPreview).toBeNull();

    pointer(jump(1), 'pointerenter');
    expect(editor.editsPreview).toEqual({ owner: 'history', edits: older.edits });
    expect(step(2).hasAttribute('data-superseded')).toBe(true);
    expect(step(1).textContent).toContain('previewing');
    expect(document.body.textContent).toContain('Newer steps are kept until your next edit.');
    expect(editor.historySteps.pill).toEqual({
      lead: 'Previewing',
      label: 'Exposure +1.00',
      tail: '· move off the list to return',
      mono: true
    });

    const list = document.body.querySelector('ul');
    if (!list) throw new Error('Missing list');
    pointer(list, 'pointerleave');
    expect(editor.editsPreview).toBeNull();
    expect(step(2).hasAttribute('data-superseded')).toBe(false);
    await destroy();
  });

  it('clears the hover preview when the list unmounts', async () => {
    api.listEditHistory.mockResolvedValue([
      { ...entry(2), edits: exposure(2) },
      { ...entry(1), edits: exposure(1) }
    ]);
    const destroy = renderLayout(layout('body', HistoryList, {}));
    await settle();

    pointer(jump(1), 'pointerenter');
    expect(editor.editsPreview?.owner).toBe('history');
    await destroy();
    expect(editor.editsPreview).toBeNull();
  });
});
