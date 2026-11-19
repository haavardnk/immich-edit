const base =
  'inline-flex h-6.5 shrink-0 items-center justify-center gap-1 rounded-full text-label whitespace-nowrap transition-colors outline-none select-none focus-visible:ring-2 focus-visible:ring-primary disabled:pointer-events-none disabled:opacity-35 touch:h-10 touch:gap-1.5 touch:px-3.5 touch:text-sm';

const shape = `${base} px-2`;

const picked =
  'aria-checked:bg-primary/20 aria-checked:text-light-950 aria-checked:ring-primary/50 aria-pressed:bg-primary/20 aria-pressed:text-light-950 aria-pressed:ring-primary/50';

const valueLook = `bg-ghost text-dark/80 ring-1 ring-hairline ring-inset hover:bg-light-950/8 hover:text-dark ${picked}`;

const denseLook =
  'bg-ghost text-dark/80 hover:text-dark hover:not-aria-checked:not-aria-pressed:bg-light-950/8 aria-checked:bg-primary aria-checked:text-light aria-pressed:bg-primary aria-pressed:text-light';

export const chipTone = {
  value: `${shape} ${valueLook}`,
  dense: `${base} px-1.5 ${denseLook}`,
  exclusion: `${shape} border border-dashed border-light-950/25 text-dark/65 hover:text-dark aria-pressed:border-solid aria-pressed:border-primary/50 aria-pressed:bg-primary/20 aria-pressed:text-light-950`,
  more: `${shape} px-2 text-primary hover:bg-ghost touch:px-2.5`
} as const;

export type ChipTone = keyof typeof chipTone;

export const chipRow = 'flex flex-wrap gap-1 touch:gap-1.5';
