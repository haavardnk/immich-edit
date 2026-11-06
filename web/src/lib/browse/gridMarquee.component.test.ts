import type { AssetGridLayout } from '#lib/browse/assetGridLayout.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { GridMarquee } from './gridMarquee.svelte';

const SIDE = 100;
const COLUMNS = 4;

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

function gridOf(count: number): AssetGridLayout {
  const boxes = Array.from({ length: count }, (_, index) => ({
    top: Math.floor(index / COLUMNS) * SIDE,
    left: (index % COLUMNS) * SIDE,
    width: SIDE,
    height: SIDE,
    row: Math.floor(index / COLUMNS),
    column: index % COLUMNS
  }));
  const rows = Array.from({ length: Math.ceil(count / COLUMNS) }, (_, row) => ({
    top: row * SIDE,
    height: SIDE,
    startIndex: row * COLUMNS,
    endIndex: Math.min(count, (row + 1) * COLUMNS)
  }));
  return { boxes, rows, height: rows.length * SIDE };
}

function pointer(type: string, x: number, y: number, init: MouseEventInit = {}): MouseEvent {
  const event = new MouseEvent(type, {
    bubbles: true,
    cancelable: true,
    clientX: x,
    clientY: y,
    button: 0,
    ...init
  });
  Object.defineProperties(event, {
    pointerId: { value: 1 },
    pointerType: { value: 'mouse' }
  });
  return event;
}

let items: AssetSummary[] = [];
let rootTop = 0;
let frames: FrameRequestCallback[] = [];
let scroller: HTMLElement;
let root: HTMLElement;
let marquee: GridMarquee;
let detach = (): void => {};

function flush(): void {
  frames.splice(0).forEach((callback) => callback(0));
}

function drag(from: [number, number], to: [number, number], init: MouseEventInit = {}): void {
  root.dispatchEvent(pointer('pointerdown', ...from, init));
  root.dispatchEvent(pointer('pointermove', ...to, init));
  flush();
}

function picked(): string[] {
  return [...selection.selected].sort();
}

beforeEach(() => {
  items = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'].map(asset);
  rootTop = 0;
  frames = [];
  vi.stubGlobal('requestAnimationFrame', (callback: FrameRequestCallback) => {
    frames.push(callback);
    return frames.length;
  });
  vi.stubGlobal('cancelAnimationFrame', () => {});
  selection.replace(['x']);
  scroller = document.createElement('div');
  root = document.createElement('div');
  scroller.append(root);
  document.body.append(scroller);
  root.getBoundingClientRect = () =>
    ({
      left: 0,
      top: rootTop,
      right: 400,
      bottom: rootTop + 600,
      width: 400,
      height: 600
    }) as DOMRect;
  marquee = new GridMarquee({
    items: () => items,
    layout: () => gridOf(items.length),
    pad: 0,
    enabled: () => true
  });
  detach = marquee.attach(root, scroller);
});

afterEach(() => {
  detach();
  scroller.remove();
  selection.clear();
  vi.unstubAllGlobals();
});

describe('GridMarquee', () => {
  it('replaces the selection with the boxed photos', () => {
    drag([50, 250], [150, 50]);
    expect(marquee.active).toBe(true);
    expect(picked()).toEqual(['a', 'b', 'e', 'f']);
    expect(marquee.covered).toBe(4);
    expect(marquee.adding).toBe(false);
    root.dispatchEvent(pointer('pointerup', 150, 50));
    expect(marquee.active).toBe(false);
    expect(picked()).toEqual(['a', 'b', 'e', 'f']);
  });

  it('adds the boxed photos while Shift is held', () => {
    drag([50, 250], [150, 50], { shiftKey: true });
    expect(picked()).toEqual(['a', 'b', 'e', 'f', 'x']);
    expect(marquee.adding).toBe(true);
  });

  it('clears the selection as soon as an empty box goes live', () => {
    drag([50, 250], [150, 350]);
    expect(marquee.active).toBe(true);
    expect(picked()).toEqual([]);
    expect(marquee.covered).toBe(0);
  });

  it('puts the old selection back on cancel', () => {
    drag([50, 250], [150, 50]);
    marquee.cancel();
    expect(marquee.active).toBe(false);
    expect(picked()).toEqual(['x']);
  });

  it('waits for the pointer to travel past a click', () => {
    drag([50, 250], [52, 251]);
    expect(marquee.active).toBe(false);
    expect(picked()).toEqual(['x']);
  });

  it('ignores presses outside the grid', () => {
    drag([50, 650], [150, 50]);
    expect(marquee.active).toBe(false);
    expect(picked()).toEqual(['x']);
  });

  it('keeps its origin on the same photo when rows are prepended above', () => {
    drag([50, 50], [150, 150]);
    expect(picked()).toEqual(['a', 'b', 'e', 'f']);
    items = [...['n1', 'n2', 'n3', 'n4'].map(asset), ...items];
    rootTop = -SIDE;
    root.dispatchEvent(pointer('pointermove', 150, 150));
    flush();
    expect(picked()).toEqual(['a', 'b', 'e', 'f']);
  });
});
