import type { LayoutChoice, LayoutMode } from '#lib/device/layoutRules.ts';

export function layoutStatus(choice: LayoutChoice, mode: LayoutMode): string {
  if (choice === 'auto') return `Using the ${mode} layout, picked for this screen.`;
  return `Using the ${mode} layout.`;
}
