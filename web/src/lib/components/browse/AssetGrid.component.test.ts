import type { MetaOutcome, MetaTarget } from '#lib/browse/assetMeta.ts';
import { gridSizeHud } from '#lib/browse/gridSizeHud.svelte.ts';
import { listenForCommands } from '#lib/commands/listen.ts';
import { browseControls } from '#lib/stores/browseControls.svelte.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { editedThumbs } from '#lib/stores/editedThumbs.svelte.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import { session } from '#lib/stores/session.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import { virtualCopies } from '#lib/stores/virtualCopies.svelte.ts';
import { asset } from '#lib/testing/assets.ts';
import ComponentHost from '#lib/testing/ComponentHost.svelte';
import type { AssetSummary } from '#lib/types/album.ts';
import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import AssetGrid from './AssetGrid.svelte';
import FilterPanel from './filters/FilterPanel.svelte';

vi.mock('$app/navigation', () => ({
  afterNavigate: (): void => {},
  goto: vi.fn()
}));

vi.mock('$app/state', () => ({
  page: { url: new URL('http://localhost/albums/a1'), params: {} }
}));

const setRating = vi.hoisted(() =>
  vi.fn(async (_target: MetaTarget, _rating: number | null): Promise<MetaOutcome> => 'saved')
);
const clipboard = vi.hoisted(() => ({ copyEditsFrom: vi.fn(), pasteEditsTo: vi.fn() }));

vi.mock('#lib/browse/copyPaste.ts', () => clipboard);

const copies = vi.hoisted(() => ({ createVirtualCopies: vi.fn() }));

vi.mock('#lib/editor/copies.ts', async (importOriginal) => ({
  ...(await importOriginal<typeof import('#lib/editor/copies.ts')>()),
  createVirtualCopies: copies.createVirtualCopies
}));

vi.mock('#lib/browse/assetMeta.ts', async (importOriginal) => ({
  ...(await importOriginal<typeof import('#lib/browse/assetMeta.ts')>()),
  setRating
}));

type GridProps = ComponentProps<typeof AssetGrid>;

const Host: Component<{ component: Component<GridProps>; props: GridProps }> = ComponentHost;

type PanelProps = ComponentProps<typeof FilterPanel>;

const PanelHost: Component<{ component: Component<PanelProps>; props: PanelProps }> = ComponentHost;

const assets = [asset('a'), asset('b'), asset('c')];

let scroller: HTMLDivElement;
let target: HTMLDivElement;
let component: Record<string, unknown> | null = null;

function render(props: Partial<GridProps> = {}): void {
  component = mount(Host, {
    target,
    props: { component: AssetGrid, props: { assets, ...props } }
  });
  flushSync();
}

function ratedIds(): [string, number | null][] {
  return setRating.mock.calls.map(([target, rating]) => [target.id, rating]);
}

function tiles(): HTMLElement[] {
  return [...scroller.querySelectorAll<HTMLElement>('[role="group"]')];
}

function selectButton(index: number): HTMLElement {
  const button = tiles()[index]?.querySelector<HTMLElement>('button[aria-pressed]');
  if (!button) throw new Error(`no select button for tile ${index}`);
  return button;
}

function click(element: HTMLElement, init: MouseEventInit = {}): void {
  element.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true, ...init }));
  flushSync();
}

function press(key: string, init: KeyboardEventInit = {}): void {
  document.body.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, ...init }));
  flushSync();
}

let stopCommands = (): void => {};

beforeEach(() => {
  stopCommands = listenForCommands();
  setRating.mockClear();
  setRating.mockResolvedValue('saved');
  clipboard.copyEditsFrom.mockClear();
  clipboard.pasteEditsTo.mockClear();
  copies.createVirtualCopies.mockReset();
  ui.closeKeybindsHelp();
  browseView.setFiltersOpen(false);
  selection.clear();
  browseView.setActive(null);
  browseControls.filters.rejected = 'any';
  scroller = document.createElement('div');
  scroller.style.overflowY = 'auto';
  target = document.createElement('div');
  scroller.appendChild(target);
  document.body.appendChild(scroller);
});

afterEach(() => {
  stopCommands();
  if (component) void unmount(component);
  component = null;
  scroller.remove();
  window.getSelection()?.removeAllRanges();
  editedThumbs.remove('b');
  browseView.setTileInfo('badges');
  browseView.setGridSize('md');
  gridSizeHud.hide();
});

describe('AssetGrid', () => {
  it('renders a tile per asset', () => {
    render();
    expect(tiles().map((tile) => tile.title)).toEqual(['a.jpg', 'b.jpg', 'c.jpg']);
  });

  it('toggles selection from a tile checkbox', () => {
    render();
    click(selectButton(1));
    expect([...selection.selected]).toEqual(['b']);
    expect(tiles()[1]?.dataset.selected).toBe('true');
    expect(document.body.textContent).toContain('1 selected');
  });

  it('extends the selection with a shift click', () => {
    render();
    click(selectButton(0));
    click(selectButton(2), { shiftKey: true });
    expect([...selection.selected].sort()).toEqual(['a', 'b', 'c']);
    expect(document.body.textContent).toContain('3 selected');
  });

  it('clears the selection on Escape', () => {
    render();
    click(selectButton(0));
    press('Escape');
    expect(selection.count).toBe(0);
    expect(document.body.textContent).not.toContain('1 selected');
  });

  it('arrow keys move a single selection', () => {
    render();
    press('ArrowRight');
    expect([...selection.selected]).toEqual(['a']);
    press('ArrowRight');
    expect([...selection.selected]).toEqual(['b']);
    expect(tiles()[1]?.dataset.selected).toBe('true');
  });

  it('the first arrow press selects the photo you left', () => {
    browseView.setActive('b');
    render();
    press('ArrowRight');
    expect([...selection.selected]).toEqual(['b']);
  });

  it('Shift+arrows grow and shrink the selection from its anchor', () => {
    render();
    press('ArrowRight');
    press('ArrowRight', { shiftKey: true });
    press('ArrowRight', { shiftKey: true });
    expect([...selection.selected].sort()).toEqual(['a', 'b', 'c']);
    press('ArrowLeft', { shiftKey: true });
    expect([...selection.selected].sort()).toEqual(['a', 'b']);
  });

  it('rating keys leave an unselected photo alone', () => {
    browseView.setActive('a');
    render();
    press('3');
    expect(setRating).not.toHaveBeenCalled();
  });

  it('makes a ticked photo the active one', () => {
    render();
    click(selectButton(1));
    expect(browseView.activeId).toBe('b');
  });

  it("creates one virtual copy per Mod+' press, ignoring key repeat", async () => {
    copies.createVirtualCopies.mockResolvedValue(undefined);
    render();
    click(selectButton(0));
    press("'", { ctrlKey: true });
    await vi.waitFor(() => expect(virtualCopies.busy).toBe(false));
    press("'", { ctrlKey: true, repeat: true });
    expect(copies.createVirtualCopies).toHaveBeenCalledExactlyOnceWith(['a']);
  });

  it('toggles the filter panel with Shift+F', async () => {
    const panel = mount(PanelHost, { target, props: { component: FilterPanel, props: {} } });
    render();
    press('F', { shiftKey: true });
    expect(browseView.filtersOpen).toBe(true);
    expect(document.querySelector('aside[aria-label="Filters"]')).not.toBeNull();
    press('F', { shiftKey: true });
    expect(browseView.filtersOpen).toBe(false);
    await unmount(panel);
  });

  it('swallows the help key while selecting all', async () => {
    let loaded = (_: boolean): void => {};
    render({ onLoadAll: () => new Promise<boolean>((done) => (loaded = done)) });
    press('?', { shiftKey: true });
    expect(ui.keybindsHelpOpen).toBe(true);
    ui.closeKeybindsHelp();

    press('a', { ctrlKey: true });
    const help = new KeyboardEvent('keydown', {
      key: '?',
      shiftKey: true,
      bubbles: true,
      cancelable: true
    });
    document.body.dispatchEvent(help);
    flushSync();
    expect(help.defaultPrevented).toBe(true);
    expect(ui.keybindsHelpOpen).toBe(false);

    loaded(true);
    await vi.waitFor(() => expect(selection.count).toBe(3));
  });

  it.each([
    ['c', 'copyEditsFrom'],
    ['v', 'pasteEditsTo']
  ] as const)('Mod+%s leaves selected text to the browser', (key, action) => {
    render();
    click(selectButton(0));
    press(key, { ctrlKey: true });
    expect(clipboard[action]).toHaveBeenCalledTimes(1);
    window.getSelection()?.selectAllChildren(document.body);
    press(key, { ctrlKey: true });
    expect(clipboard[action]).toHaveBeenCalledTimes(1);
  });

  it.each([
    ['first', 0, ['c']],
    ['second', 1, ['b']]
  ])('arrows move from a photo still ticked after unticking the %s', (_name, untick, selected) => {
    render();
    click(selectButton(0));
    click(selectButton(1));
    click(selectButton(untick));
    press('ArrowRight');
    expect([...selection.selected]).toEqual(selected);
  });

  it.each([
    ['moves the tick to the next photo', 'saved', ['b'], 'b'],
    ['moves on even when Immich refuses the rating', 'failed', ['b'], 'b'],
    ['stays when the rating is not written', 'skipped', ['a'], 'a']
  ] as const)('Shift+3 on one ticked photo %s', async (_name, outcome, selected, active) => {
    setRating.mockResolvedValue(outcome);
    render();
    click(selectButton(0));
    press('#', { code: 'Digit3', shiftKey: true });
    await vi.waitFor(() => expect(ratedIds()).toEqual([['a', 3]]));
    await Promise.resolve();
    flushSync();
    expect([...selection.selected]).toEqual(selected);
    expect(browseView.activeId).toBe(active);
  });

  it('Shift+3 rates every ticked photo and keeps the selection', () => {
    render();
    click(selectButton(0));
    click(selectButton(1));
    press('#', { code: 'Digit3', shiftKey: true });
    expect(ratedIds()).toEqual([
      ['a', 3],
      ['b', 3]
    ]);
    expect([...selection.selected].sort()).toEqual(['a', 'b']);
  });

  it('keeps photos owned by another Immich user read-only', () => {
    session.set({ id: 'me', email: 'me@x', name: 'Me', is_admin: false, auth_kind: 'password' });
    render({
      assets: [
        { ...asset('a'), ownerId: 'me' },
        { ...asset('b'), ownerId: 'partner' }
      ]
    });
    const starsDisabled = tiles().map(
      (tile) => tile.querySelector<HTMLButtonElement>('[role="radio"]')?.disabled
    );
    expect(starsDisabled).toEqual([false, true]);
    click(selectButton(0));
    const bar = document.querySelector('[aria-label="Selection actions"]');
    const tags = (): HTMLButtonElement | null | undefined =>
      bar?.querySelector<HTMLButtonElement>('[aria-label="Tags"]');
    expect(tags()?.disabled).toBe(false);
    click(selectButton(1));
    expect(tags()?.disabled).toBe(true);
    expect(bar?.querySelector<HTMLButtonElement>('[role="radio"]')?.disabled).toBe(true);
    expect(bar?.querySelector<HTMLButtonElement>('[aria-label="Albums"]')?.disabled).toBe(false);
  });

  it('marks edited photos in badges and name-and-date, never in hover-only', () => {
    editedThumbs.upsert('b', 'h1', 'u1');
    render();
    const marked = (): boolean[] =>
      tiles().map((tile) => tile.querySelector('[role="img"][aria-label="Edited"]') !== null);

    expect(marked()).toEqual([false, true, false]);
    browseView.setTileInfo('full');
    flushSync();
    expect(marked()).toEqual([false, true, false]);
    browseView.setTileInfo('none');
    flushSync();
    expect(marked()).toEqual([false, false, false]);
  });

  it('steps the thumbnail size from the keyboard and shows the size HUD', () => {
    render();
    const status = (): Element | null =>
      document.querySelector('[role="status"]:has([data-testid="grid-size-hud"])');
    const live = Array.from(document.querySelectorAll('[role="status"]'));
    expect(status()).toBeNull();

    press('=');
    expect(browseView.gridSize).toBe('lg');
    expect(live).toContain(status());
    const hud = document.querySelector('[data-testid="grid-size-hud"]');
    expect(hud?.textContent).toContain('220 px');
    expect(hud?.textContent).toContain('Ctrl scroll · pinch');

    press('-');
    press('-');
    expect(browseView.gridSize).toBe('sm');
    expect(document.querySelector('[data-testid="grid-size-hud"]')?.textContent).toContain(
      '110 px'
    );
  });
});
