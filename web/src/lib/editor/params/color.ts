import {
  BW_CHANNELS,
  neutralDcp,
  type BwChannel,
  type BwTint,
  type Edits
} from '#lib/types/edits.ts';
import { param } from './define';
import { BALANCE_GRADIENT, HUE_GRADIENT, saturationGradient } from './gradients';
import { AMOUNT, HUE, PERCENT } from './range';
import type { ParamDef, SectionDef } from './types';

const CHANNEL_HUES: Record<BwChannel, number> = {
  red: 0,
  yellow: 60,
  green: 120,
  aqua: 180,
  blue: 240,
  magenta: 300
};

function bwMix(channel: BwChannel): ParamDef {
  const label = channel.charAt(0).toUpperCase() + channel.slice(1);
  const hue = CHANNEL_HUES[channel];
  return param({
    id: `bw_${channel}`,
    label,
    history: `B&W ${label}`,
    synonyms: ['black and white', 'B&W', 'monochrome'],
    ...PERCENT,
    gradient: `linear-gradient(to right, hsl(${hue}, 0%, 15%), hsl(${hue}, 70%, 55%), hsl(${hue}, 0%, 90%))`,
    get: (edits) => edits.color.bw.mix[channel],
    set: (edits, value) => {
      edits.color.bw.mix[channel] = value;
    }
  });
}

export const BW_MIX: readonly ParamDef[] = BW_CHANNELS.map(bwMix);

export const BW_TINT_REGIONS = [
  { key: 'shadows', label: 'Shadows' },
  { key: 'highlights', label: 'Highlights' }
] as const;

type TintRegion = (typeof BW_TINT_REGIONS)[number]['key'];

function tintOf(edits: Edits, region: TintRegion): BwTint {
  return edits.color.bw[region];
}

function bwTint(region: TintRegion, label: string): readonly ParamDef[] {
  const synonyms = ['black and white', 'B&W', 'split tone', 'tint'];
  return [
    param({
      id: `bw_${region}_hue`,
      label: `${label} Hue`,
      history: `B&W ${label} Hue`,
      synonyms,
      ...HUE,
      gradient: HUE_GRADIENT,
      get: (edits) => tintOf(edits, region).hue,
      set: (edits, value) => {
        tintOf(edits, region).hue = value;
      }
    }),
    param({
      id: `bw_${region}_sat`,
      label: `${label} Saturation`,
      history: `B&W ${label} Saturation`,
      synonyms,
      ...AMOUNT,
      liveGradient: (edits) => saturationGradient(Math.round(tintOf(edits, region).hue)),
      get: (edits) => tintOf(edits, region).sat,
      set: (edits, value) => {
        tintOf(edits, region).sat = value;
      }
    })
  ];
}

const BW_TINTS: readonly ParamDef[] = BW_TINT_REGIONS.flatMap(({ key, label }) =>
  bwTint(key, label)
);

export const bwBalance = param({
  id: 'bw_balance',
  label: 'Balance',
  history: 'B&W Tint Balance',
  synonyms: ['black and white', 'B&W', 'split tone'],
  ...PERCENT,
  gradient: BALANCE_GRADIENT,
  get: (edits) => edits.color.bw.balance,
  set: (edits, value) => {
    edits.color.bw.balance = value;
  }
});

export const lutAmount = param({
  id: 'lut_amount',
  label: 'Amount',
  history: 'LUT Amount',
  synonyms: ['LUT', 'look', 'cube'],
  ...AMOUNT,
  defaultValue: 100,
  get: (edits) => edits.color.lut_3d.amount,
  set: (edits, value) => {
    edits.color.lut_3d.amount = value;
  }
});

export const COLOR_PARAMS: readonly ParamDef[] = [...BW_MIX, ...BW_TINTS, bwBalance, lutAmount];

export const BW: SectionDef = {
  id: 'bw',
  title: 'B&W',
  history: 'Black & White',
  rows: [BW_MIX, [...BW_TINTS, bwBalance]],
  clear: (edits) => {
    edits.color.bw.enabled = false;
  }
};

export const LUT: SectionDef = {
  id: 'lut',
  title: 'LUT',
  rows: [[lutAmount]],
  clear: (edits) => {
    edits.color.lut_3d.lut_id = null;
  }
};

export const DCP: SectionDef = {
  id: 'dcp',
  title: 'Camera Profile',
  rows: [],
  clear: (edits) => {
    edits.color.dcp = neutralDcp();
  }
};
