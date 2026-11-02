import { accentOptions } from '#lib/theme/families/accentOptions.ts';
import type { Scheme, ThemeDefinition, ThemeFamily } from '#lib/theme/types.ts';

const ACCENTS = [
  'blue',
  'cyan',
  'magenta',
  'purple',
  'orange',
  'yellow',
  'green',
  'teal',
  'red'
] as const;

type Accent = (typeof ACCENTS)[number];

type Role =
  | 'bg'
  | 'bgDark'
  | 'bgDark1'
  | 'bgHighlight'
  | 'fgGutter'
  | 'terminalBlack'
  | 'comment'
  | 'fgDark'
  | 'fg';

interface Style {
  id: string;
  name: string;
  scheme: Scheme;
  colors: Readonly<Record<Accent | Role, string>>;
}

const STORM_ACCENTS: Readonly<Record<Accent, string>> = {
  blue: '#7aa2f7',
  cyan: '#7dcfff',
  magenta: '#bb9af7',
  purple: '#9d7cd8',
  orange: '#ff9e64',
  yellow: '#e0af68',
  green: '#9ece6a',
  teal: '#1abc9c',
  red: '#f7768e'
};

const STORM_SURFACES = {
  bgHighlight: '#292e42',
  fgGutter: '#3b4261',
  terminalBlack: '#414868',
  comment: '#565f89',
  fgDark: '#a9b1d6',
  fg: '#c0caf5'
};

const STYLES: readonly Style[] = [
  {
    id: 'tokyo-night',
    name: 'Tokyo Night',
    scheme: 'dark',
    colors: {
      ...STORM_ACCENTS,
      ...STORM_SURFACES,
      bg: '#1a1b26',
      bgDark: '#16161e',
      bgDark1: '#0c0e14'
    }
  },
  {
    id: 'tokyo-night-storm',
    name: 'Tokyo Night Storm',
    scheme: 'dark',
    colors: {
      ...STORM_ACCENTS,
      ...STORM_SURFACES,
      bg: '#24283b',
      bgDark: '#1f2335',
      bgDark1: '#1b1e2d'
    }
  },
  {
    id: 'tokyo-night-moon',
    name: 'Tokyo Night Moon',
    scheme: 'dark',
    colors: {
      blue: '#82aaff',
      cyan: '#86e1fc',
      magenta: '#c099ff',
      purple: '#fca7ea',
      orange: '#ff966c',
      yellow: '#ffc777',
      green: '#c3e88d',
      teal: '#4fd6be',
      red: '#ff757f',
      bg: '#222436',
      bgDark: '#1e2030',
      bgDark1: '#191b29',
      bgHighlight: '#2f334d',
      fgGutter: '#3b4261',
      terminalBlack: '#444a73',
      comment: '#636da6',
      fgDark: '#828bb8',
      fg: '#c8d3f5'
    }
  },
  {
    id: 'tokyo-night-day',
    name: 'Tokyo Night Day',
    scheme: 'light',
    colors: {
      blue: '#2e7de9',
      cyan: '#007197',
      magenta: '#9854f1',
      purple: '#7847bd',
      orange: '#b15c00',
      yellow: '#8c6c3e',
      green: '#587539',
      teal: '#118c74',
      red: '#f52a65',
      bg: '#e1e2e7',
      bgDark: '#d0d5e3',
      bgDark1: '#c1c9df',
      bgHighlight: '#c4c8da',
      fgGutter: '#a8aecb',
      terminalBlack: '#a1a6c5',
      comment: '#848cb5',
      fgDark: '#6172b0',
      fg: '#3760bf'
    }
  }
];

function theme(style: Style): ThemeDefinition {
  const pick = (name: Accent | Role): string => style.colors[name];
  return {
    id: style.id,
    name: style.name,
    scheme: style.scheme,
    palette: {
      crust: pick('bgDark1'),
      mantle: pick('bgDark'),
      base: pick('bg'),
      surface0: pick('bgHighlight'),
      surface1: pick('fgGutter'),
      surface2: pick('terminalBlack'),
      overlay: pick('comment'),
      subtext: pick('fgDark'),
      text: pick('fg'),
      success: pick('green'),
      warning: pick('yellow'),
      danger: pick('red'),
      info: pick('blue')
    },
    accents: Object.fromEntries(ACCENTS.map((accent) => [accent, pick(accent)]))
  };
}

export const tokyoNight: ThemeFamily = {
  id: 'tokyo-night',
  name: 'Tokyo Night',
  defaultAccent: 'blue',
  accents: accentOptions(ACCENTS),
  themes: STYLES.map(theme)
};
