import { describe, expect, it } from 'vitest';
import {
  centroid,
  distance,
  offsetOf,
  pointerTable,
  travelOf,
  velocityOf,
  type PointerSample
} from '#lib/gestures/pointers.ts';

function at(pointerId: number, clientX: number, clientY: number, timeStamp: number): PointerSample {
  return { pointerId, pointerType: 'touch', clientX, clientY, timeStamp };
}

describe('pointerTable', () => {
  it('tracks pointers by id with start, current, type and order', () => {
    const table = pointerTable();
    table.down({ ...at(7, 10, 20, 0), pointerType: 'pen' });
    table.down(at(3, 50, 60, 5));
    table.move(at(7, 15, 25, 16));
    expect(table.count()).toBe(2);
    expect(table.first()).toMatchObject({
      id: 7,
      type: 'pen',
      start: { x: 10, y: 20 },
      startTime: 0,
      current: { x: 15, y: 25 },
      time: 16
    });
    expect(table.second()?.id).toBe(3);
    expect(table.list().map((pointer) => pointer.id)).toEqual([7, 3]);
  });

  it('records the release sample and forgets the pointer on up', () => {
    const table = pointerTable();
    table.down(at(1, 0, 0, 0));
    const released = table.up(at(1, 30, 40, 50));
    expect(released?.current).toEqual({ x: 30, y: 40 });
    expect(released && travelOf(released)).toBe(50);
    expect(table.has(1)).toBe(false);
    expect(table.first()).toBeNull();
  });

  it('drops a cancelled pointer without recording a sample', () => {
    const table = pointerTable();
    table.down(at(1, 0, 0, 0));
    const cancelled = table.cancel(1);
    expect(cancelled?.current).toEqual({ x: 0, y: 0 });
    expect(table.count()).toBe(0);
    expect(table.cancel(1)).toBeNull();
  });

  it('ignores moves and ups for unknown pointers', () => {
    const table = pointerTable();
    expect(table.move(at(9, 1, 1, 1))).toBeNull();
    expect(table.up(at(9, 1, 1, 1))).toBeNull();
    expect(table.get(9)).toBeNull();
  });

  it('measures velocity only over the last 100 ms', () => {
    const table = pointerTable();
    table.down(at(1, 0, 0, 0));
    table.move(at(1, 100, 0, 50));
    table.move(at(1, 110, 10, 160));
    table.move(at(1, 120, 30, 200));
    const pointer = table.move(at(1, 130, 50, 250));
    expect(pointer?.samples.map((sample) => sample.time)).toEqual([160, 200, 250]);
    expect(pointer && velocityOf(pointer)).toEqual({ x: 20 / 90, y: 40 / 90 });
  });

  it('reports no velocity after the pointer rests past the window', () => {
    const table = pointerTable();
    table.down(at(1, 0, 0, 0));
    table.move(at(1, 100, 0, 50));
    const released = table.up(at(1, 100, 0, 300));
    expect(released && velocityOf(released)).toEqual({ x: 0, y: 0 });
  });

  it('takes a custom velocity window', () => {
    const table = pointerTable(1000);
    table.down(at(1, 0, 0, 0));
    table.move(at(1, 100, 0, 50));
    const pointer = table.move(at(1, 100, 0, 500));
    expect(pointer && velocityOf(pointer).x).toBeCloseTo(0.2);
  });

  it('replaces a stale entry for a reused id and moves it to the back', () => {
    const table = pointerTable();
    table.down(at(1, 0, 0, 0));
    table.down(at(2, 5, 5, 0));
    table.down(at(1, 50, 60, 100));
    expect(table.count()).toBe(2);
    expect(table.first()?.id).toBe(2);
    expect(table.second()).toMatchObject({ id: 1, start: { x: 50, y: 60 }, startTime: 100 });
  });

  it('rebases a pointer at its current position', () => {
    const table = pointerTable();
    table.down(at(1, 0, 0, 0));
    table.move(at(1, 40, 0, 50));
    const rebased = table.rebase(1);
    expect(rebased).toMatchObject({ start: { x: 40, y: 0 }, startTime: 50 });
    expect(rebased && travelOf(rebased)).toBe(0);
    expect(rebased && velocityOf(rebased)).toEqual({ x: 0, y: 0 });
    expect(table.rebase(9)).toBeNull();
  });

  it('clears every pointer', () => {
    const table = pointerTable();
    table.down(at(1, 0, 0, 0));
    table.down(at(2, 0, 0, 0));
    table.clear();
    expect(table.count()).toBe(0);
  });
});

describe('pointer geometry', () => {
  it('finds the centroid and distance of two points', () => {
    expect(centroid({ x: 0, y: 0 }, { x: 30, y: 40 })).toEqual({ x: 15, y: 20 });
    expect(distance({ x: 0, y: 0 }, { x: 30, y: 40 })).toBe(50);
  });

  it('measures the offset from the start point', () => {
    const table = pointerTable();
    table.down(at(1, 10, 10, 0));
    const pointer = table.move(at(1, 7, 14, 10));
    expect(pointer && offsetOf(pointer)).toEqual({ x: -3, y: 4 });
    expect(pointer && travelOf(pointer)).toBe(5);
  });
});
