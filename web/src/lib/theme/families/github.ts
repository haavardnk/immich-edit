import { mix } from '#lib/theme/color.ts';
import { accentOptions } from '#lib/theme/families/accentOptions.ts';
import type { Scheme, ThemeDefinition, ThemeFamily } from '#lib/theme/types.ts';

const ACCENTS = ['blue', 'purple', 'pink', 'red', 'orange', 'yellow', 'green'] as const;

type Accent = (typeof ACCENTS)[number];

type Role =
  | 'canvasInset'
  | 'canvasDefault'
  | 'surface0'
  | 'surface1'
  | 'surface2'
  | 'overlay'
  | 'fgMuted'
  | 'fgDefault';

interface Variant {
  id: string;
  name: string;
  scheme: Scheme;
  colors: Readonly<Record<Accent | Role, string>>;
}

const VARIANTS: readonly Variant[] = [
  {
    id: 'github-light',
    name: 'GitHub Light',
    scheme: 'light',
    colors: {
      canvasInset: '#f6f8fa',
      canvasDefault: '#ffffff',
      surface0: '#eaeef2',
      surface1: '#d0d7de',
      surface2: '#afb8c1',
      overlay: '#8c959f',
      fgMuted: '#656d76',
      fgDefault: '#1f2328',
      blue: '#0969da',
      purple: '#8250df',
      pink: '#bf3989',
      red: '#d1242f',
      orange: '#bc4c00',
      yellow: '#9a6700',
      green: '#1a7f37'
    }
  },
  {
    id: 'github-dark',
    name: 'GitHub Dark',
    scheme: 'dark',
    colors: {
      canvasInset: '#010409',
      canvasDefault: '#0d1117',
      surface0: '#161b22',
      surface1: '#21262d',
      surface2: '#30363d',
      overlay: '#6e7681',
      fgMuted: '#848d97',
      fgDefault: '#e6edf3',
      blue: '#2f81f7',
      purple: '#a371f7',
      pink: '#db61a2',
      red: '#f85149',
      orange: '#db6d28',
      yellow: '#d29922',
      green: '#3fb950'
    }
  },
  {
    id: 'github-dark-dimmed',
    name: 'GitHub Dark Dimmed',
    scheme: 'dark',
    colors: {
      canvasInset: '#1c2128',
      canvasDefault: '#22272e',
      surface0: '#2d333b',
      surface1: '#373e47',
      surface2: '#444c56',
      overlay: '#636e7b',
      fgMuted: '#768390',
      fgDefault: '#adbac7',
      blue: '#539bf5',
      purple: '#986ee2',
      pink: '#c96198',
      red: '#e5534b',
      orange: '#cc6b2c',
      yellow: '#c69026',
      green: '#57ab5a'
    }
  }
];

function theme(variant: Variant): ThemeDefinition {
  const pick = (name: Accent | Role): string => variant.colors[name];
  return {
    id: variant.id,
    name: variant.name,
    scheme: variant.scheme,
    palette: {
      crust: pick('canvasInset'),
      mantle: mix(pick('canvasInset'), pick('canvasDefault'), 0.5),
      base: pick('canvasDefault'),
      surface0: pick('surface0'),
      surface1: pick('surface1'),
      surface2: pick('surface2'),
      overlay: pick('overlay'),
      subtext: pick('fgMuted'),
      text: pick('fgDefault'),
      success: pick('green'),
      warning: pick('yellow'),
      danger: pick('red'),
      info: pick('blue')
    },
    accents: Object.fromEntries(ACCENTS.map((accent) => [accent, pick(accent)]))
  };
}

export const github: ThemeFamily = {
  id: 'github',
  name: 'GitHub',
  defaultAccent: 'blue',
  accents: accentOptions(ACCENTS),
  themes: VARIANTS.map(theme)
};
