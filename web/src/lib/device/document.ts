import type { LayoutMode } from '#lib/device/layoutRules.ts';

export function showLayout(mode: LayoutMode, wide: boolean): void {
  const root = document.documentElement;
  root.dataset.layout = mode;
  root.toggleAttribute('data-wide', wide);
}
