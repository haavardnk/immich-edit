export type Thresholds = {
  tapSlop: number;
  clickTravel: number;
  doubleTapMs: number;
  doubleTapSlop: number;
  longPressMs: number;
  panSlop: number;
  swipeMin: number;
  swipeDominance: number;
  flingVelocity: number;
  pinchSlop: number;
  velocityWindowMs: number;
};

export const DEFAULT_THRESHOLDS: Readonly<Thresholds> = {
  tapSlop: 10,
  clickTravel: 5,
  doubleTapMs: 300,
  doubleTapSlop: 30,
  longPressMs: 500,
  panSlop: 10,
  swipeMin: 60,
  swipeDominance: 2,
  flingVelocity: 0.5,
  pinchSlop: 0.05,
  velocityWindowMs: 100
};

export function thresholds(overrides: Partial<Thresholds> = {}): Thresholds {
  return { ...DEFAULT_THRESHOLDS, ...overrides };
}
