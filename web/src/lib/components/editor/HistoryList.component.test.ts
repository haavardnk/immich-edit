import type { EditHistoryEntry } from '#lib/api/edits.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import {
  layout,
  layoutControls,
  renderLayout,
  settle,
  type ControlRef
} from '#lib/testing/layoutParity.ts';
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
const ENTRY = 'li button:not([aria-controls])';

function entry(id: number): EditHistoryEntry {
  return {
    id,
    manifest_hash: `h${id}`,
    deleted: false,
    edits: null,
    created_at: '2026-01-01T10:00:00Z',
    action: null
  };
}

function click(selector: string): void {
  const target = document.body.querySelector<HTMLButtonElement>(selector);
  if (!target) throw new Error(`Missing ${selector}`);
  target.click();
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
});

afterEach(() => {
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
  it('loads the history once per open', async () => {
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
    expect(api.listEditHistory).toHaveBeenCalledTimes(2);
    await destroy();
  });

  it('closes after a restore', async () => {
    const restore = vi.spyOn(editor, 'restoreHistoryEntry').mockResolvedValue();
    const destroy = renderLayout(layout('popover', HistoryPopover, {}));
    openPopover();
    await settle();

    click(`${POPOVER} ${ENTRY}`);
    await settle();
    expect(restore).toHaveBeenCalledWith(2);
    await closed();
    await destroy();
  });
});

describe('HistoryList', () => {
  it('reloads and calls onRestored after restoring an entry', async () => {
    const restore = vi.spyOn(editor, 'restoreHistoryEntry').mockResolvedValue();
    const onRestored = vi.fn();
    const destroy = renderLayout(layout('body', HistoryList, { onRestored }));
    await settle();
    expect(api.listEditHistory).toHaveBeenCalledTimes(1);

    click(ENTRY);
    await settle();
    expect(restore).toHaveBeenCalledWith(2);
    expect(api.listEditHistory).toHaveBeenCalledTimes(2);
    expect(onRestored).toHaveBeenCalledTimes(1);
    await destroy();
  });
});
