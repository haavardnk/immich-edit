import type { LayoutMode } from '#lib/device/layoutRules.ts';

type LayoutState = {
  mode: LayoutMode;
  wide: boolean;
  landscape: boolean;
  phone: boolean;
  hasKeyboard: boolean;
};

const DESKTOP: LayoutState = {
  mode: 'desktop',
  wide: false,
  landscape: false,
  phone: false,
  hasKeyboard: false
};

export const layoutState: LayoutState = { ...DESKTOP };

export function resetLayout(next: Partial<LayoutState> = {}): void {
  Object.assign(layoutState, DESKTOP, next);
}

export const fakeLayout = {
  get mode(): LayoutMode {
    return layoutState.mode;
  },
  get wide(): boolean {
    return layoutState.wide;
  },
  get landscape(): boolean {
    return layoutState.landscape;
  },
  get compact(): boolean {
    return layoutState.mode === 'touch' && !layoutState.wide;
  },
  get phone(): boolean {
    return layoutState.phone;
  },
  get hasKeyboard(): boolean {
    return layoutState.hasKeyboard;
  }
};
