import { mix } from '#lib/theme/color.ts';
import { accentOptions } from '#lib/theme/families/accentOptions.ts';
import type { Scheme, ThemeDefinition, ThemeFamily } from '#lib/theme/types.ts';

const ACCENTS = ['red', 'green', 'yellow', 'blue', 'purple', 'aqua', 'orange'] as const;

type Accent = (typeof ACCENTS)[number];

type Role = 'bg0h' | 'bg0' | 'bg1' | 'bg2' | 'bg3' | 'bg4' | 'fg4' | 'fg1';

interface Mode {
  id: string;
  name: string;
  scheme: Scheme;
  colors: Readonly<Record<Accent | Role, string>>;
}

const MODES: readonly Mode[] = [
  {
    id: 'gruvbox-dark',
    name: 'Gruvbox Dark',
    scheme: 'dark',
    colors: {
      bg0h: '#1d2021',
      bg0: '#282828',
      bg1: '#3c3836',
      bg2: '#504945',
      bg3: '#665c54',
      bg4: '#7c6f64',
      fg4: '#a89984',
      fg1: '#ebdbb2',
      red: '#fb4934',
      green: '#b8bb26',
      yellow: '#fabd2f',
      blue: '#83a598',
      purple: '#d3869b',
      aqua: '#8ec07c',
      orange: '#fe8019'
    }
  },
  {
    id: 'gruvbox-light',
    name: 'Gruvbox Light',
    scheme: 'light',
    colors: {
      bg0h: '#f9f5d7',
      bg0: '#fbf1c7',
      bg1: '#ebdbb2',
      bg2: '#d5c4a1',
      bg3: '#bdae93',
      bg4: '#a89984',
      fg4: '#7c6f64',
      fg1: '#3c3836',
      red: '#9d0006',
      green: '#79740e',
      yellow: '#b57614',
      blue: '#076678',
      purple: '#8f3f71',
      aqua: '#427b58',
      orange: '#af3a03'
    }
  }
];

function theme(mode: Mode): ThemeDefinition {
  const pick = (name: Accent | Role): string => mode.colors[name];
  return {
    id: mode.id,
    name: mode.name,
    scheme: mode.scheme,
    palette: {
      crust: pick('bg0h'),
      mantle: mix(pick('bg0h'), pick('bg0'), 0.5),
      base: pick('bg0'),
      surface0: pick('bg1'),
      surface1: pick('bg2'),
      surface2: pick('bg3'),
      overlay: pick('bg4'),
      subtext: pick('fg4'),
      text: pick('fg1'),
      success: pick('green'),
      warning: pick('yellow'),
      danger: pick('red'),
      info: pick('blue')
    },
    accents: Object.fromEntries(ACCENTS.map((accent) => [accent, pick(accent)]))
  };
}

export const gruvbox: ThemeFamily = {
  id: 'gruvbox',
  name: 'Gruvbox',
  defaultAccent: 'orange',
  accents: accentOptions(ACCENTS),
  themes: MODES.map(theme)
};
