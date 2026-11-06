import type { AssetGridLayout } from '#lib/browse/assetGridLayout.ts';
import { gestureArbiter } from '#lib/gestures/arbiter.ts';
import type { PointerSample } from '#lib/gestures/pointers.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import { asset } from '#lib/testing/assets.ts';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { TouchRange } from './touchRange.svelte';

const SIDE = 100;
const COLUMNS = 4;
const HOLD = 500;

const ITEMS = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'].map((id) => asset(id));

function grid(): AssetGridLayout {
  const boxes = ITEMS.map((_, index) => ({
    top: Math.floor(index / COLUMNS) * SIDE,
    left: (index % COLUMNS) * SIDE,
    width: SIDE - 10,
    height: SIDE - 10,
    row: Math.floor(index / COLUMNS),
    column: index % COLUMNS
  }));
  const rows = [0, 1].map((row) => ({
    top: row * SIDE,
    height: SIDE - 10,
    startIndex: row * COLUMNS,
    endIndex: (row + 1) * COLUMNS
  }));
  return { boxes, rows, height: 2 * SIDE };
}

function at(x: number, y: number, timeStamp: number): PointerSample {
  return { pointerId: 1, pointerType: 'touch', clientX: x, clientY: y, timeStamp };
}

let range: TouchRange;
let arbiter: ReturnType<typeof gestureArbiter>;

beforeEach(() => {
  vi.useFakeTimers();
  vi.stubGlobal('requestAnimationFrame', () => 0);
  vi.stubGlobal('cancelAnimationFrame', () => {});
  const root = document.createElement('div');
  root.getBoundingClientRect = () =>
    ({ left: 0, top: 0, right: 400, bottom: 200, width: 400, height: 200 }) as DOMRect;
  range = new TouchRange({
    items: () => ITEMS,
    layout: grid,
    pad: 0,
    root: () => root,
    scroller: () => null
  });
  arbiter = gestureArbiter([range.recognizer(() => true)]);
});

afterEach(() => {
  range.cancel();
  selection.clear();
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

function press(x: number, y: number): void {
  arbiter.down(at(x, y, 0));
  vi.advanceTimersByTime(HOLD);
}

describe('TouchRange', () => {
  it('enters select mode and picks the photo under a long press', () => {
    press(20, 20);
    expect(selection.mode).toBe(true);
    expect([...selection.selected]).toEqual(['a']);
    expect(range.dragging).toBe(true);
    expect(range.target).toBeNull();
  });

  it('previews the range while the finger moves on and selects it on release', () => {
    press(20, 20);
    arbiter.move(at(220, 120, 600));
    expect(range.target).toBe('g');
    expect(range.added).toBe(6);
    expect([...selection.selected]).toEqual(['a']);
    arbiter.up(at(220, 120, 700));
    expect([...selection.selected].sort()).toEqual(['a', 'b', 'c', 'd', 'e', 'f', 'g']);
    expect(range.dragging).toBe(false);
    expect(range.target).toBeNull();
  });

  it('writes the selection once when the held finger stays on the pressed photo', () => {
    press(20, 20);
    const picked = selection.selected;
    arbiter.move(at(26, 24, 600));
    arbiter.up(at(26, 24, 700));
    expect(selection.selected).toBe(picked);
    expect([...selection.selected]).toEqual(['a']);
  });

  it('ranges from an already picked photo without dropping it', () => {
    selection.replace(['b', 'h']);
    press(120, 20);
    expect([...selection.selected].sort()).toEqual(['b', 'h']);
    arbiter.move(at(320, 20, 600));
    arbiter.up(at(320, 20, 700));
    expect([...selection.selected].sort()).toEqual(['b', 'c', 'd', 'h']);
  });

  it('does nothing when the press lands in a gap', () => {
    press(95, 95);
    expect(selection.mode).toBe(false);
    expect(selection.count).toBe(0);
    expect(range.dragging).toBe(false);
    expect(arbiter.locked()).toBeNull();
  });

  it('keeps the pressed photo but drops the range on cancel', () => {
    press(20, 20);
    arbiter.move(at(320, 20, 600));
    expect(range.added).toBe(3);
    arbiter.cancel(1);
    expect(range.target).toBeNull();
    expect([...selection.selected]).toEqual(['a']);
  });
});
