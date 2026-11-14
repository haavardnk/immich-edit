export type LayoutChoice = 'auto' | 'desktop' | 'touch';
export type LayoutMode = 'desktop' | 'touch';

export interface LayoutInput {
  width: number;
  height: number;
  coarse: boolean;
  choice: LayoutChoice;
}

export interface Layout {
  mode: LayoutMode;
  wide: boolean;
  landscape: boolean;
  compact: boolean;
  phone: boolean;
}

export const LAYOUT_CHOICES: readonly LayoutChoice[] = ['auto', 'desktop', 'touch'];

const NARROW_WIDTH = 768;
const TOUCH_SHORT_SIDE = 1100;
const WIDE_SHORT_SIDE = 600;
const WIDE_MIN_WIDTH = 1024;

function autoMode(width: number, height: number, coarse: boolean): LayoutMode {
  if (width < NARROW_WIDTH) return 'touch';
  if (coarse && Math.min(width, height) <= TOUCH_SHORT_SIDE) return 'touch';
  return 'desktop';
}

export function resolveLayout({ width, height, coarse, choice }: LayoutInput): Layout {
  const mode = choice === 'auto' ? autoMode(width, height, coarse) : choice;
  const landscape = width > height;
  const tabletSized = Math.min(width, height) >= WIDE_SHORT_SIDE;
  const wide = mode === 'touch' && landscape && width >= WIDE_MIN_WIDTH && tabletSized;
  return {
    mode,
    wide,
    landscape,
    compact: mode === 'touch' && !wide,
    phone: mode === 'touch' && !tabletSized
  };
}
