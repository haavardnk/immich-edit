export type WhiteBalance =
  | { mode: 'relative'; temp: number; tint: number }
  | { mode: 'absolute'; kelvin: number; tint: number };

export interface Temperature {
  kelvin: number;
  tint: number;
}

export interface AsShotWhite extends Temperature {
  raw: boolean;
}

export const KELVIN_MIN = 2000;
export const KELVIN_MAX = 50000;
export const TINT_LIMIT = 150;
export const RELATIVE_LIMIT = 100;
const RELATIVE_TEMP_STOPS = 1.5;
const RELATIVE_TINT_SCALE = 0.9;

function clamp(value: number, limit: number): number {
  return Math.min(limit, Math.max(-limit, value));
}

function clampKelvin(kelvin: number): number {
  return Math.min(KELVIN_MAX, Math.max(KELVIN_MIN, kelvin));
}

export function asShotWhiteBalance(): WhiteBalance {
  return { mode: 'relative', temp: 0, tint: 0 };
}

export function whiteBalanceIsAsShot(wb: WhiteBalance): boolean {
  return wb.mode === 'relative' && wb.temp === 0 && wb.tint === 0;
}

export function resolveTemperature(wb: WhiteBalance, asShot: Temperature): Temperature {
  if (wb.mode === 'absolute') {
    return { kelvin: clampKelvin(wb.kelvin), tint: clamp(wb.tint, TINT_LIMIT) };
  }
  return {
    kelvin: clampKelvin(asShot.kelvin * 2 ** ((wb.temp * RELATIVE_TEMP_STOPS) / 100)),
    tint: clamp(asShot.tint + wb.tint * RELATIVE_TINT_SCALE, TINT_LIMIT)
  };
}

export function relativeShift(
  wb: WhiteBalance,
  asShot: Temperature
): { temp: number; tint: number } {
  if (wb.mode === 'relative') return { temp: wb.temp, tint: wb.tint };
  return {
    temp: clamp((Math.log2(wb.kelvin / asShot.kelvin) * 100) / RELATIVE_TEMP_STOPS, RELATIVE_LIMIT),
    tint: clamp((wb.tint - asShot.tint) / RELATIVE_TINT_SCALE, RELATIVE_LIMIT)
  };
}

export function absoluteWhiteBalance(white: Temperature, asShot: Temperature | null): WhiteBalance {
  const kelvin = Math.round(clampKelvin(white.kelvin));
  const tint = Math.round(clamp(white.tint, TINT_LIMIT));
  if (asShot && kelvin === asShot.kelvin && tint === asShot.tint) return asShotWhiteBalance();
  return { mode: 'absolute', kelvin, tint };
}

export function relativeWhiteBalance(temp: number, tint: number): WhiteBalance {
  return {
    mode: 'relative',
    temp: clamp(temp, RELATIVE_LIMIT),
    tint: clamp(tint, RELATIVE_LIMIT)
  };
}
