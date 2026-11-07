const textItem =
  'flex h-6 items-center justify-center rounded-sm text-[10px] font-medium text-dark/65 transition-colors hover:bg-ghost hover:text-dark';

export const segmented = {
  text: {
    group: 'flex h-7 gap-0.5 rounded-sm bg-ghost p-0.5',
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
      'flex h-11 items-center justify-center gap-1.5 rounded-[10px] px-2 text-[13px] font-medium whitespace-nowrap opacity-70 transition-colors hover:opacity-100 aria-checked:bg-current/15 aria-checked:font-semibold aria-checked:opacity-100',
    icon: 'hidden shrink-0 @[17rem]:block'
  },
  row: {
    group: 'flex gap-1 rounded-lg bg-gray-100 p-1 dark:bg-gray-800 h-10',
    radio:
      'flex h-full flex-1 items-center justify-center rounded-md px-1.5 text-xs font-medium whitespace-nowrap text-dark/65 transition-colors hover:bg-ghost hover:text-dark aria-checked:bg-primary aria-checked:text-light'
  }
} as const;
