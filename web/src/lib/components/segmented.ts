const textItem =
  'flex h-6 items-center justify-center rounded-sm text-micro font-medium text-dark/65 transition-colors hover:bg-ghost hover:text-dark touch:h-11 touch:rounded-md touch:text-label';

export const segmented = {
  text: {
    group: 'flex h-7 gap-0.5 rounded-sm bg-ghost p-0.5 touch:h-12 touch:rounded-lg',
    radio: `${textItem} aria-checked:bg-primary aria-checked:text-light`,
    tab: `${textItem} aria-selected:bg-primary aria-selected:text-light`
  },
  tool: {
    group: 'h-5 overflow-hidden rounded-sm ring-1 ring-light-950/10',
    swatch:
      'h-5 rounded-sm ring-1 ring-light-950/10 transition-shadow hover:ring-light-950/35 aria-checked:ring-2 aria-checked:ring-primary'
  },
  photo: {
    group: '@container grid h-12 gap-0.5 rounded-xl bg-current/10 p-0.5',
    radio:
      'flex h-11 items-center justify-center gap-1.5 rounded-xl px-2 text-label font-medium whitespace-nowrap opacity-70 transition-colors hover:opacity-100 aria-checked:bg-current/15 aria-checked:font-semibold aria-checked:opacity-100',
    icon: 'hidden shrink-0 @[17rem]:block'
  }
} as const;
