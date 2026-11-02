import { mix } from '#lib/theme/color.ts';
import { accentOptions } from '#lib/theme/families/accentOptions.ts';
import type { Scheme, ThemeDefinition, ThemeFamily } from '#lib/theme/types.ts';

const ACCENTS = ['green', 'aqua', 'blue', 'purple', 'red', 'orange', 'yellow'] as const;

type Accent = (typeof ACCENTS)[number];

type Background = 'bgDim' | 'bg0' | 'bg1' | 'bg3' | 'bg4' | 'bg5';

type Role = Background | 'grey0' | 'grey2' | 'fg';

interface Variant {
  id: string;
  name: string;
  scheme: Scheme;
  surfaces: readonly [Background, Background, Background];
  colors: Readonly<Record<Accent | Role, string>>;
}

const VARIANTS: readonly Variant[] = [
  {
    id: 'everforest-dark',
    name: 'Everforest Dark',
    scheme: 'dark',
    surfaces: ['bg1', 'bg3', 'bg5'],
    colors: {
      bgDim: '#232a2e',
      bg0: '#2d353b',
      bg1: '#343f44',
      bg3: '#475258',
      bg4: '#4f585e',
      bg5: '#56635f',
      grey0: '#7a8478',
      grey2: '#9da9a0',
      fg: '#d3c6aa',
      green: '#a7c080',
      aqua: '#83c092',
      blue: '#7fbbb3',
      purple: '#d699b6',
      red: '#e67e80',
      orange: '#e69875',
      yellow: '#dbbc7f'
    }
  },
  {
    id: 'everforest-light',
    name: 'Everforest Light',
    scheme: 'light',
    surfaces: ['bg3', 'bg4', 'bg5'],
    colors: {
      bgDim: '#efebd4',
      bg0: '#fdf6e3',
      bg1: '#f4f0d9',
      bg3: '#e6e2cc',
      bg4: '#e0dcc7',
      bg5: '#bdc3af',
      grey0: '#a6b0a0',
      grey2: '#829181',
      fg: '#5c6a72',
      green: '#8da101',
      aqua: '#35a77c',
      blue: '#3a94c5',
      purple: '#df69ba',
      red: '#f85552',
      orange: '#f57d26',
      yellow: '#dfa000'
    }
  }
];

function theme(variant: Variant): ThemeDefinition {
  const pick = (name: Accent | Role): string => variant.colors[name];
  const [surface0, surface1, surface2] = variant.surfaces;
  return {
    id: variant.id,
    name: variant.name,
    scheme: variant.scheme,
    palette: {
      crust: pick('bgDim'),
      mantle: mix(pick('bgDim'), pick('bg0'), 0.5),
      base: pick('bg0'),
      surface0: pick(surface0),
      surface1: pick(surface1),
      surface2: pick(surface2),
      overlay: pick('grey0'),
      subtext: pick('grey2'),
      text: pick('fg'),
      success: pick('green'),
      warning: pick('yellow'),
      danger: pick('red'),
      info: pick('blue')
    },
    accents: Object.fromEntries(ACCENTS.map((accent) => [accent, pick(accent)]))
  };
}

export const everforest: ThemeFamily = {
  id: 'everforest',
  name: 'Everforest',
  defaultAccent: 'green',
  accents: accentOptions(ACCENTS),
  themes: VARIANTS.map(theme)
};
