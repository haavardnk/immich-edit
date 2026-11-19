const EDGE_BAND = 56;
const EDGE_MAX_SPEED = 24;
const EDGE_MIN_SPEED = 2;

export type EdgeScroller = { track: (y: number) => void; stop: () => void };

export function edgeSpeed(y: number, top: number, bottom: number, band = EDGE_BAND): number {
  const reach = Math.min(band, (bottom - top) / 2);
  if (reach <= 0) return 0;
  const depth = Math.max(reach - (y - top), y - (bottom - reach), 0) / reach;
  if (depth === 0) return 0;
  const speed = Math.max(EDGE_MIN_SPEED, Math.min(1, depth) * EDGE_MAX_SPEED);
  return y - top < bottom - y ? -speed : speed;
}

export function edgeScroller(
  scroller: () => HTMLElement | null,
  onScroll: () => void
): EdgeScroller {
  let y: number | null = null;
  let frame: number | null = null;

  function step(): void {
    frame = null;
    const element = scroller();
    if (!element || y === null) return;
    const box = element.getBoundingClientRect();
    const speed = edgeSpeed(y, box.top, box.bottom);
    if (speed === 0) return;
    const before = element.scrollTop;
    element.scrollTop = before + speed;
    if (element.scrollTop !== before) onScroll();
    frame = requestAnimationFrame(step);
  }

  return {
    track(next) {
      y = next;
      if (frame === null) frame = requestAnimationFrame(step);
    },
    stop() {
      y = null;
      if (frame !== null) cancelAnimationFrame(frame);
      frame = null;
    }
  };
}
