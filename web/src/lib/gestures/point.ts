export type Box = { left: number; top: number; width: number; height: number };

export type Point = { x: number; y: number };

type ClientPoint = { clientX: number; clientY: number };

function boxOf(target: Element | Box): Box {
  return 'getBoundingClientRect' in target ? target.getBoundingClientRect() : target;
}

export function elementPoint(e: ClientPoint, target: Element | Box): Point {
  const box = boxOf(target);
  return { x: e.clientX - box.left, y: e.clientY - box.top };
}

export function normalizedPoint(e: ClientPoint, target: Element | Box): Point {
  const box = boxOf(target);
  return {
    x: (e.clientX - box.left) / Math.max(1, box.width),
    y: (e.clientY - box.top) / Math.max(1, box.height)
  };
}

function unit(value: number): number {
  return Math.min(1, Math.max(0, value));
}

export function clampedPoint(e: ClientPoint, target: Element | Box): Point {
  const point = normalizedPoint(e, target);
  return { x: unit(point.x), y: unit(point.y) };
}

export function centredPoint(e: ClientPoint, target: Element | Box): Point {
  const box = boxOf(target);
  return {
    x: e.clientX - box.left - box.width / 2,
    y: e.clientY - box.top - box.height / 2
  };
}
