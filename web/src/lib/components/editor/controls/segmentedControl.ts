const segmentedItemClass =
  'flex h-6 items-center justify-center rounded-sm text-[10px] font-medium text-dark/65 transition-colors hover:bg-ghost hover:text-dark';

export const segmentedControlClass = 'flex h-7 gap-0.5 rounded-sm bg-ghost p-0.5';
export const segmentedRadioItemClass = `${segmentedItemClass} aria-checked:bg-primary aria-checked:text-light`;
export const segmentedTabItemClass = `${segmentedItemClass} aria-selected:bg-primary aria-selected:text-light`;
export const compactSegmentedControlClass = 'h-5 overflow-hidden rounded-sm ring-1 ring-white/10';
export const compactSegmentedSwatchItemClass =
  'h-5 rounded-sm ring-1 ring-white/10 transition-shadow hover:ring-white/35 aria-checked:ring-2 aria-checked:ring-primary';
