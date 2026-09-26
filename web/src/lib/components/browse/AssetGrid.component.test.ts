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

function press(key: string): void {
  document.body.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true }));
  flushSync();
}

beforeEach(() => {
  selection.clear();
  browseView.setActive(null);
  browseControls.rejected = 'any';
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

  it('moves the active asset with the arrow keys', () => {
    render();
    press('ArrowRight');
    expect(browseView.activeId).toBe('a');
    press('ArrowRight');
    expect(browseView.activeId).toBe('b');
  });
});
