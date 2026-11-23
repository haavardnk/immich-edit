import { twMerge } from '#lib/theme/tailwindMerge.ts';

const unpicked =
  'not-aria-checked:not-aria-pressed:not-data-[state=active]:bg-ghost not-aria-checked:not-aria-pressed:not-data-[state=active]:text-dark/80 hover:not-aria-checked:not-aria-pressed:not-data-[state=active]:bg-light-950/8 hover:not-aria-checked:not-aria-pressed:not-data-[state=active]:text-dark';

const picked =
  'aria-checked:bg-primary/22 aria-checked:text-light-950 aria-pressed:bg-primary/22 aria-pressed:text-light-950 data-[state=active]:bg-primary/22 data-[state=active]:text-light-950';

const shape =
  'relative inline-flex h-7 shrink-0 items-center justify-center gap-1 rounded-full px-2.5 text-xs font-medium whitespace-nowrap transition-colors duration-fast outline-none select-none focus-visible:ring-2 focus-visible:ring-primary disabled:pointer-events-none disabled:opacity-35 touch:h-9 touch:gap-1.5 touch:px-3.5 touch:text-sm touch:before:absolute touch:before:inset-x-0 touch:before:-inset-y-1';

export const chip = `${shape} ${unpicked} ${picked}`;

export const chipMore = `${shape} text-primary hover:bg-ghost`;

export const chipFill = twMerge(chip, 'min-w-0 grow px-1.5 touch:px-2.5');

export const chipRow = 'flex flex-wrap gap-1 touch:gap-1.5';

export const chipRemovable =
  'inline-flex h-7 max-w-full min-w-0 shrink-0 items-center gap-1 rounded-full bg-primary/22 pl-2.5 text-xs font-medium whitespace-nowrap text-light-950 touch:h-9 touch:gap-1.5 touch:pl-3.5 touch:text-sm';
