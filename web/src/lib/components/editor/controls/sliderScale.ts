export interface SliderScale {
  toPosition: (value: number) => number;
  fromPosition: (position: number) => number;
}

export const SCALE_POSITIONS = 1000;

export function miredScale(minKelvin: number, maxKelvin: number): SliderScale {
  const hi = 1e6 / minKelvin;
  const lo = 1e6 / maxKelvin;
  return {
    toPosition: (kelvin) => ((hi - 1e6 / kelvin) / (hi - lo)) * SCALE_POSITIONS,
    fromPosition: (position) => Math.round(1e6 / (hi - (position / SCALE_POSITIONS) * (hi - lo)))
  };
}
