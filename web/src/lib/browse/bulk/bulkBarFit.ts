export const BAR_CONTAINER = '@container/bulk';

export const ROOMY_ONLY = 'hidden @min-[56rem]/bulk:flex';
export const TIGHT_ONLY = '@min-[56rem]/bulk:hidden';
export const NAME_WIDTH = 'max-w-32 @min-[56rem]/bulk:max-w-40';

const ONE_ROW_GAP = '@min-[42rem]/bulk:gap-1';

export const ONE_ROW = {
  row: 'grid grid-cols-[minmax(0,1fr)_auto] px-1.5 @min-[42rem]/bulk:flex @min-[42rem]/bulk:px-3',
  head: 'ps-1.5 @min-[42rem]/bulk:shrink-0 @min-[42rem]/bulk:ps-0',
  end: `col-start-2 row-start-1 ${ONE_ROW_GAP}`,
  group: ONE_ROW_GAP,
  tools: `col-span-2 flex-wrap border-t border-hairline pt-1 @min-[42rem]/bulk:flex-nowrap ${ONE_ROW_GAP} @min-[42rem]/bulk:border-t-0 @min-[42rem]/bulk:pt-0`,
  divider: 'hidden @min-[42rem]/bulk:block'
} as const;
