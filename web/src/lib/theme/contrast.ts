import { composite, contrastRatio, hexToOklch, mix, oklchToHex } from './color';
import type { Palette, PaletteSlot, SurfaceSlot } from './types';

type ContrastRole = PaletteSlot | 'accent';

type ContrastBackground = SurfaceSlot | 'accentTint';

interface ContrastPair {
  foreground: ContrastRole;
  background: ContrastBackground;
  minimum: number;
  alpha: number;
}

type ContrastColors = Record<ContrastRole, string>;

const TEXT_MINIMUM = 4.5;
const GRAPHIC_MINIMUM = 3;
const SECONDARY_TEXT_ALPHA = 0.65;
const NUDGE_STEP = 0.005;
const ACCENT_TINT = 0.1;
const MAX_PASSES = 8;

function pairs(
  foreground: ContrastRole,
  backgrounds: readonly ContrastBackground[],
  minimum: number,
  alpha = 1
): ContrastPair[] {
  return backgrounds.map((background) => ({ foreground, background, minimum, alpha }));
}

export const CONTRAST_PAIRS: readonly ContrastPair[] = [
  ...pairs('text', ['crust', 'mantle', 'base', 'surface0', 'surface1'], TEXT_MINIMUM),
  ...pairs('text', ['crust', 'mantle', 'base'], TEXT_MINIMUM, SECONDARY_TEXT_ALPHA),
  ...pairs('subtext', ['mantle', 'base', 'surface0'], TEXT_MINIMUM),
  ...pairs('accent', ['base', 'mantle', 'crust', 'accentTint'], TEXT_MINIMUM),
  ...pairs('success', ['base', 'mantle'], TEXT_MINIMUM),
  ...pairs('warning', ['base', 'mantle'], TEXT_MINIMUM),
  ...pairs('danger', ['base', 'mantle'], TEXT_MINIMUM),
  ...pairs('info', ['base', 'mantle'], TEXT_MINIMUM),
  ...pairs('overlay', ['base'], GRAPHIC_MINIMUM)
];

export function backgroundColor(colors: ContrastColors, background: ContrastBackground): string {
  return background === 'accentTint'
    ? mix(colors.mantle, colors.accent, ACCENT_TINT)
    : colors[background];
}

export function pairContrast(foreground: string, background: string, alpha = 1): number {
  return contrastRatio(composite(foreground, background, alpha), background);
}

export function nudge(foreground: string, background: string, minimum: number, alpha = 1): string {
  if (pairContrast(foreground, background, alpha) >= minimum) return foreground;
  const color = hexToOklch(foreground);
  const direction = hexToOklch(background).l > color.l ? -1 : 1;
  for (let l = color.l; l >= 0 && l <= 1; l += direction * NUDGE_STEP) {
    const candidate = oklchToHex({ ...color, l });
    if (pairContrast(candidate, background, alpha) >= minimum) return candidate;
  }
  return oklchToHex({ ...color, l: direction < 0 ? 0 : 1 });
}

export function pairName({ foreground, background, alpha }: ContrastPair): string {
  const tier = alpha < 1 ? `@${Math.round(alpha * 100)}` : '';
  return `${foreground}${tier}/${background}`;
}

interface GateResult {
  palette: Palette;
  accent: string;
  nudges: string[];
}

export function gate(palette: Palette, accent: string): GateResult {
  const colors: ContrastColors = { ...palette, accent };
  const nudges: string[] = [];
  for (const pair of CONTRAST_PAIRS) {
    const { foreground, background, minimum, alpha } = pair;
    const before = colors[foreground];
    for (let pass = 0; pass < MAX_PASSES; pass += 1) {
      const current = colors[foreground];
      const next = nudge(current, backgroundColor(colors, background), minimum, alpha);
      if (next === current) break;
      colors[foreground] = next;
    }
    if (colors[foreground] !== before) nudges.push(pairName(pair));
  }
  const { accent: gated, ...rest } = colors;
  return { palette: rest, accent: gated, nudges };
}
