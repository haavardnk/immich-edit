import { BW_CHANNELS, type BwChannel } from '#lib/types/edits.ts';
import { param } from './define';
import { AMOUNT, PERCENT } from './range';
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

export const bwBalance = param({
  id: 'bw_balance',
  label: 'Balance',
  history: 'B&W Tint Balance',
  synonyms: ['black and white', 'B&W', 'split tone'],
  ...PERCENT,
  gradient: 'var(--gradient-grading-balance)',
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

export const COLOR_PARAMS: readonly ParamDef[] = [...BW_MIX, bwBalance, lutAmount];

export const BW: SectionDef = {
  id: 'bw',
  title: 'B&W',
  rows: [BW_MIX, [bwBalance]]
};

export const LUT: SectionDef = {
  id: 'lut',
  title: 'LUT',
  rows: [[lutAmount]]
};
