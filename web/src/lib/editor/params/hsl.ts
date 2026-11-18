import { HSL_BAND_HUES, HSL_BAND_NAMES, type Edits, type HslBand } from '#lib/types/edits.ts';
import { param } from './define';
import { luminanceGradient, saturationGradient } from './gradients';
import { PERCENT } from './range';
import type { ParamDef, SectionDef } from './types';

type Channel = keyof HslBand;

const CHANNELS: readonly { key: Channel; label: string }[] = [
  { key: 'hue', label: 'Hue' },
  { key: 'sat', label: 'Saturation' },
  { key: 'lum', label: 'Luminance' }
];

function bandOf(edits: Edits, band: number): HslBand | undefined {
  return edits.color.hsl.bands[band];
}

function shiftedHue(edits: Edits, band: number): number {
  return ((HSL_BAND_HUES[band] ?? 0) + (bandOf(edits, band)?.hue ?? 0) + 360) % 360;
}

function shiftedSat(edits: Edits, band: number): number {
  return Math.max(0, Math.min(100, ((bandOf(edits, band)?.sat ?? 0) + 100) / 2));
}

function hueGradient(band: number): string {
  const hue = HSL_BAND_HUES[band] ?? 0;
  return `linear-gradient(to right, hsl(${(hue - 100 + 360) % 360}, 50%, 50%), hsl(${hue}, 50%, 50%), hsl(${(hue + 100) % 360}, 50%, 50%))`;
}

function liveGradient(channel: Channel, band: number): ((edits: Edits) => string) | undefined {
  if (channel === 'sat') return (edits) => saturationGradient(shiftedHue(edits, band));
  if (channel === 'lum') {
    return (edits) => luminanceGradient(shiftedHue(edits, band), shiftedSat(edits, band));
  }
  return undefined;
}

function hslParam(band: number, { key, label }: { key: Channel; label: string }): ParamDef {
  const name = HSL_BAND_NAMES[band] ?? '';
  return param({
    id: `hsl_${name.toLowerCase()}_${key}`,
    label,
    history: `${name} ${label}`,
    synonyms: ['HSL', 'color mixer', name.toLowerCase()],
    ...PERCENT,
    gradient: key === 'hue' ? hueGradient(band) : undefined,
    liveGradient: liveGradient(key, band),
    get: (edits) => bandOf(edits, band)?.[key] ?? 0,
    set: (edits, value) => {
      const target = bandOf(edits, band);
      if (target) target[key] = value;
    }
  });
}

export const HSL_BAND_PARAMS: readonly (readonly ParamDef[])[] = HSL_BAND_NAMES.map((_, band) =>
  CHANNELS.map((channel) => hslParam(band, channel))
);

export const HSL_PARAMS: readonly ParamDef[] = HSL_BAND_PARAMS.flat();

export const HSL: SectionDef = {
  id: 'hsl',
  title: 'HSL',
  rows: HSL_BAND_PARAMS
};
