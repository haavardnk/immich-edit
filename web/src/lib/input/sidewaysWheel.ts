import { held } from './modifiers.svelte.ts';

export function scrollSideways(e: WheelEvent): void {
  const row = e.currentTarget;
  if (!(row instanceof HTMLElement) || e.deltaX !== 0 || e.deltaY === 0 || held('zoom', e)) return;
  const before = row.scrollLeft;
  row.scrollLeft += e.deltaY;
  if (row.scrollLeft !== before) e.preventDefault();
}
