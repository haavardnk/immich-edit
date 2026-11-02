import type { Palette, Scheme, ThemeDefinition, ThemeFamily } from '#lib/theme/types.ts';

const ACCENTS = [
  'rosewater',
  'flamingo',
  'pink',
  'mauve',
  'red',
  'maroon',
  'peach',
  'yellow',
  'green',
  'teal',
  'sky',
  'sapphire',
  'blue',
  'lavender'
] as const;

type Accent = (typeof ACCENTS)[number];

type Role =
  | 'text'
  | 'subtext1'
  | 'overlay1'
  | 'surface2'
  | 'surface1'
  | 'surface0'
  | 'base'
  | 'mantle'
  | 'crust';

interface Flavour {
  id: string;
  name: string;
  scheme: Scheme;
  colors: Readonly<Record<Accent | Role, string>>;
}

const FLAVOURS: readonly Flavour[] = [
  {
    id: 'catppuccin-latte',
    name: 'Catppuccin Latte',
    scheme: 'light',
    colors: {
      rosewater: '#dc8a78',
      flamingo: '#dd7878',
      pink: '#ea76cb',
      mauve: '#8839ef',
      red: '#d20f39',
      maroon: '#e64553',
      peach: '#fe640b',
      yellow: '#df8e1d',
      green: '#40a02b',
      teal: '#179299',
      sky: '#04a5e5',
      sapphire: '#209fb5',
      blue: '#1e66f5',
      lavender: '#7287fd',
      text: '#4c4f69',
      subtext1: '#5c5f77',
      overlay1: '#8c8fa1',
      surface2: '#acb0be',
      surface1: '#bcc0cc',
      surface0: '#ccd0da',
      base: '#eff1f5',
      mantle: '#e6e9ef',
      crust: '#dce0e8'
    }
  },
  {
    id: 'catppuccin-frappe',
    name: 'Catppuccin Frappé',
    scheme: 'dark',
    colors: {
      rosewater: '#f2d5cf',
      flamingo: '#eebebe',
      pink: '#f4b8e4',
      mauve: '#ca9ee6',
      red: '#e78284',
      maroon: '#ea999c',
      peach: '#ef9f76',
      yellow: '#e5c890',
      green: '#a6d189',
      teal: '#81c8be',
      sky: '#99d1db',
      sapphire: '#85c1dc',
      blue: '#8caaee',
      lavender: '#babbf1',
      text: '#c6d0f5',
      subtext1: '#b5bfe2',
      overlay1: '#838ba7',
      surface2: '#626880',
      surface1: '#51576d',
      surface0: '#414559',
      base: '#303446',
      mantle: '#292c3c',
      crust: '#232634'
    }
  },
  {
    id: 'catppuccin-macchiato',
    name: 'Catppuccin Macchiato',
    scheme: 'dark',
    colors: {
      rosewater: '#f4dbd6',
      flamingo: '#f0c6c6',
      pink: '#f5bde6',
      mauve: '#c6a0f6',
      red: '#ed8796',
      maroon: '#ee99a0',
      peach: '#f5a97f',
      yellow: '#eed49f',
      green: '#a6da95',
      teal: '#8bd5ca',
      sky: '#91d7e3',
      sapphire: '#7dc4e4',
      blue: '#8aadf4',
      lavender: '#b7bdf8',
      text: '#cad3f5',
      subtext1: '#b8c0e0',
      overlay1: '#8087a2',
      surface2: '#5b6078',
      surface1: '#494d64',
      surface0: '#363a4f',
      base: '#24273a',
      mantle: '#1e2030',
      crust: '#181926'
    }
  },
  {
    id: 'catppuccin-mocha',
    name: 'Catppuccin Mocha',
    scheme: 'dark',
    colors: {
      rosewater: '#f5e0dc',
      flamingo: '#f2cdcd',
      pink: '#f5c2e7',
      mauve: '#cba6f7',
      red: '#f38ba8',
      maroon: '#eba0ac',
      peach: '#fab387',
      yellow: '#f9e2af',
      green: '#a6e3a1',
      teal: '#94e2d5',
      sky: '#89dceb',
      sapphire: '#74c7ec',
      blue: '#89b4fa',
      lavender: '#b4befe',
      text: '#cdd6f4',
      subtext1: '#bac2de',
      overlay1: '#7f849c',
      surface2: '#585b70',
      surface1: '#45475a',
      surface0: '#313244',
      base: '#1e1e2e',
      mantle: '#181825',
      crust: '#11111b'
    }
  }
];

function theme(flavour: Flavour): ThemeDefinition {
  const pick = (name: Accent | Role): string => flavour.colors[name];
  const palette: Palette = {
    crust: pick('crust'),
    mantle: pick('mantle'),
    base: pick('base'),
    surface0: pick('surface0'),
    surface1: pick('surface1'),
    surface2: pick('surface2'),
    overlay: pick('overlay1'),
    subtext: pick('subtext1'),
    text: pick('text'),
    success: pick('green'),
    warning: pick('yellow'),
    danger: pick('red'),
    info: pick('blue')
  };
  return {
    id: flavour.id,
    name: flavour.name,
    scheme: flavour.scheme,
    palette,
    accents: Object.fromEntries(ACCENTS.map((accent) => [accent, pick(accent)]))
  };
}

export const catppuccin: ThemeFamily = {
  id: 'catppuccin',
  name: 'Catppuccin',
  defaultAccent: 'mauve',
  accents: ACCENTS.map((id) => ({ id, name: id.charAt(0).toUpperCase() + id.slice(1) })),
  themes: FLAVOURS.map(theme)
};
