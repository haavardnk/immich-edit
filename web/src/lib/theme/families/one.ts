import { mix } from '#lib/theme/color.ts';
import { accentOptions } from '#lib/theme/families/accentOptions.ts';
import type { Scheme, ThemeDefinition, ThemeFamily } from '#lib/theme/types.ts';

const ACCENTS = ['blue', 'purple', 'cyan', 'green', 'yellow', 'orange', 'red'] as const;

type Accent = (typeof ACCENTS)[number];

type Role = 'crust' | 'mantle' | 'base' | 'surface0' | 'surface1' | 'mono3' | 'mono2' | 'mono1';

interface Variant {
  id: string;
  name: string;
  scheme: Scheme;
  colors: Readonly<Record<Accent | Role, string>>;
}

const VARIANTS: readonly Variant[] = [
  {
    id: 'one-dark',
    name: 'One Dark',
    scheme: 'dark',
    colors: {
      crust: '#181a1f',
      mantle: '#21252b',
      base: '#282c34',
      surface0: '#353b45',
      surface1: '#3e4451',
      mono3: '#5c6370',
      mono2: '#828997',
      mono1: '#abb2bf',
      blue: '#61afef',
      purple: '#c678dd',
      cyan: '#56b6c2',
      green: '#98c379',
      yellow: '#e5c07b',
      orange: '#d19a66',
      red: '#e06c75'
    }
  },
  {
    id: 'one-light',
    name: 'One Light',
    scheme: 'light',
    colors: {
      crust: '#eaeaeb',
      mantle: '#f2f2f2',
      base: '#fafafa',
      surface0: '#e5e5e6',
      surface1: '#dbdbdc',
      mono3: '#a0a1a7',
      mono2: '#696c77',
      mono1: '#383a42',
      blue: '#4078f2',
      purple: '#a626a4',
      cyan: '#0184bc',
      green: '#50a14f',
      yellow: '#cb7701',
      orange: '#b76b01',
      red: '#e45649'
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
      crust: pick('crust'),
      mantle: pick('mantle'),
      base: pick('base'),
      surface0: pick('surface0'),
      surface1: pick('surface1'),
      surface2: mix(pick('surface1'), pick('mono3'), 0.5),
      overlay: pick('mono3'),
      subtext: pick('mono2'),
      text: pick('mono1'),
      success: pick('green'),
      warning: pick('yellow'),
      danger: pick('red'),
      info: pick('blue')
    },
    accents: Object.fromEntries(ACCENTS.map((accent) => [accent, pick(accent)]))
  };
}

export const one: ThemeFamily = {
  id: 'one',
  name: 'One',
  defaultAccent: 'blue',
  accents: accentOptions(ACCENTS),
  themes: VARIANTS.map(theme)
};
