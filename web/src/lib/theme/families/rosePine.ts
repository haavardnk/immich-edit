import { shiftLightness } from '#lib/theme/color.ts';
import { accentOptions } from '#lib/theme/families/accentOptions.ts';
import type { Palette, Scheme, ThemeDefinition, ThemeFamily } from '#lib/theme/types.ts';

const ACCENTS = ['love', 'gold', 'rose', 'pine', 'foam', 'iris'] as const;

type Accent = (typeof ACCENTS)[number];

type Role =
  'base' | 'surface' | 'overlay' | 'muted' | 'subtle' | 'text' | 'highlightMed' | 'highlightHigh';

interface Variant {
  id: string;
  name: string;
  scheme: Scheme;
  colors: Readonly<Record<Accent | Role, string>>;
}

const MANTLE_SHIFT = -0.025;
const CRUST_SHIFT = -0.05;

const VARIANTS: readonly Variant[] = [
  {
    id: 'rose-pine',
    name: 'Rosé Pine',
    scheme: 'dark',
    colors: {
      base: '#191724',
      surface: '#1f1d2e',
      overlay: '#26233a',
      muted: '#6e6a86',
      subtle: '#908caa',
      text: '#e0def4',
      love: '#eb6f92',
      gold: '#f6c177',
      rose: '#ebbcba',
      pine: '#31748f',
      foam: '#9ccfd8',
      iris: '#c4a7e7',
      highlightMed: '#403d52',
      highlightHigh: '#524f67'
    }
  },
  {
    id: 'rose-pine-moon',
    name: 'Rosé Pine Moon',
    scheme: 'dark',
    colors: {
      base: '#232136',
      surface: '#2a273f',
      overlay: '#393552',
      muted: '#6e6a86',
      subtle: '#908caa',
      text: '#e0def4',
      love: '#eb6f92',
      gold: '#f6c177',
      rose: '#ea9a97',
      pine: '#3e8fb0',
      foam: '#9ccfd8',
      iris: '#c4a7e7',
      highlightMed: '#44415a',
      highlightHigh: '#56526e'
    }
  },
  {
    id: 'rose-pine-dawn',
    name: 'Rosé Pine Dawn',
    scheme: 'light',
    colors: {
      base: '#faf4ed',
      surface: '#fffaf3',
      overlay: '#f2e9e1',
      muted: '#9893a5',
      subtle: '#797593',
      text: '#575279',
      love: '#b4637a',
      gold: '#ea9d34',
      rose: '#d7827e',
      pine: '#286983',
      foam: '#56949f',
      iris: '#907aa9',
      highlightMed: '#dfdad9',
      highlightHigh: '#cecacd'
    }
  }
];

function surfaces(variant: Variant): Pick<Palette, 'surface0' | 'surface1' | 'surface2'> {
  const { surface, overlay, highlightMed, highlightHigh } = variant.colors;
  return variant.scheme === 'dark'
    ? { surface0: surface, surface1: overlay, surface2: highlightMed }
    : { surface0: overlay, surface1: highlightMed, surface2: highlightHigh };
}

function theme(variant: Variant): ThemeDefinition {
  const pick = (name: Accent | Role): string => variant.colors[name];
  return {
    id: variant.id,
    name: variant.name,
    scheme: variant.scheme,
    palette: {
      crust: shiftLightness(pick('base'), CRUST_SHIFT),
      mantle: shiftLightness(pick('base'), MANTLE_SHIFT),
      base: pick('base'),
      ...surfaces(variant),
      overlay: pick('muted'),
      subtext: pick('subtle'),
      text: pick('text'),
      success: pick('pine'),
      warning: pick('gold'),
      danger: pick('love'),
      info: pick('foam')
    },
    accents: Object.fromEntries(ACCENTS.map((accent) => [accent, pick(accent)]))
  };
}

export const rosePine: ThemeFamily = {
  id: 'rose-pine',
  name: 'Rosé Pine',
  defaultAccent: 'iris',
  accents: accentOptions(ACCENTS),
  themes: VARIANTS.map(theme)
};
