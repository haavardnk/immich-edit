import { flushSync, mount, unmount, type Component, type ComponentProps } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import AssetGrid from './AssetGrid.svelte';
import ComponentHost from '$lib/testing/ComponentHost.svelte';
import { selection } from '$lib/stores/selection.svelte';
import { browseView } from '$lib/stores/browseView.svelte';
import { browseControls } from '$lib/stores/browseControls.svelte';
import type { AssetSummary } from '$lib/types/album';

vi.mock('$app/navigation', () => ({
  afterNavigate: (): void => {},
  goto: vi.fn()
}));

vi.mock('$app/state', () => ({
  page: { url: new URL('http://localhost/albums/a1'), params: {} }
}));

const rateAsset = vi.hoisted(() => vi.fn(async (): Promise<boolean> => true));

vi.mock('$lib/cull', async (importOriginal) => ({
  ...(await importOriginal<typeof import('$lib/cull')>()),
  rateAsset
}));

type GridProps = ComponentProps<typeof AssetGrid>;

const Host: Component<{ component: Component<GridProps>; props: GridProps }> = ComponentHost;

function asset(id: string): AssetSummary {
  return {
    id,
    originalFileName: `${id}.jpg`,
    type: 'IMAGE',
    fileCreatedAt: null,
    updatedAt: null,
    checksum: null,
    isFavorite: false,
    exifInfo: null,
    tags: []
  };
}

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

beforeEach(() => {
  rateAsset.mockClear();
  rateAsset.mockResolvedValue(true);
  selection.clear();
  browseView.setActive(null);
  browseControls.excludeRejected = false;
  scroller = document.createElement('div');
  scroller.style.overflowY = 'auto';
  target = document.createElement('div');
  scroller.appendChild(target);
  document.body.appendChild(scroller);
});

afterEach(() => {
  if (component) void unmount(component);
  component = null;
  scroller.remove();
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
    expect(rateAsset).not.toHaveBeenCalled();
  });

  it('makes a ticked photo the active one', () => {
    render();
    click(selectButton(1));
    expect(browseView.activeId).toBe('b');
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
    ['moves the tick to the next photo', true, ['b'], 'b'],
    ['stays when the rating fails to save', false, ['a'], 'a']
  ])('Shift+3 on one ticked photo %s', async (_name, ok, selected, active) => {
    rateAsset.mockResolvedValue(ok);
    render();
    click(selectButton(0));
    press('#', { code: 'Digit3', shiftKey: true });
    await vi.waitFor(() => expect(rateAsset).toHaveBeenCalledWith('a', 3));
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
    expect(rateAsset.mock.calls).toEqual([
      ['a', 3],
      ['b', 3]
    ]);
    expect([...selection.selected].sort()).toEqual(['a', 'b']);
  });
});
