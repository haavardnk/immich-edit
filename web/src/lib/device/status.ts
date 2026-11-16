import {
  LAYOUT_CHOICE_LABELS,
  type LayoutChoice,
  type LayoutMode
} from '#lib/device/layoutRules.ts';

export function layoutHint(choice: LayoutChoice, mode: LayoutMode): string | null {
  return choice === 'auto' ? `${LAYOUT_CHOICE_LABELS[mode]} on this screen` : null;
}
