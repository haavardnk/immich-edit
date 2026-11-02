import { mix } from '#lib/theme/color.ts';
import { accentOptions } from '#lib/theme/families/accentOptions.ts';
import type { Scheme, ThemeDefinition, ThemeFamily } from '#lib/theme/types.ts';

const ACCENTS = ['purple', 'pink', 'cyan', 'green', 'orange', 'yellow', 'red'] as const;

type Accent = (typeof ACCENTS)[number];

type Role =
  | 'crust'
  | 'mantle'
  | 'background'
  | 'surface0'
  | 'surface1'
  | 'surface2'
  | 'comment'
  | 'foreground';

interface Variant {
  id: string;
  name: string;
  scheme: Scheme;
  colors: Readonly<Record<Accent | Role, string>>;
}

const SUBTEXT_TOWARD_COMMENT = 0.4;

const VARIANTS: readonly Variant[] = [
  {
    id: 'dracula',
    name: 'Dracula',
    scheme: 'dark',
    colors: {
      crust: '#191a21',
      mantle: '#21222c',
      background: '#282a36',
      surface0: '#343746',
      surface1: '#424450',
      surface2: '#44475a',
      comment: '#6272a4',
      foreground: '#f8f8f2',
      purple: '#bd93f9',
      pink: '#ff79c6',
      cyan: '#8be9fd',
      green: '#50fa7b',
      orange: '#ffb86c',
      yellow: '#f1fa8c',
      red: '#ff5555'
    }
  },
  {
    id: 'alucard',
    name: 'Alucard',
    scheme: 'light',
    colors: {
      crust: '#dedccf',
      mantle: '#ece9df',
      background: '#fffbeb',
      surface0: '#efeddc',
      surface1: '#ceccc0',
      surface2: '#bcbab3',
      comment: '#6c664b',
      foreground: '#1f1f1f',
      purple: '#644ac9',
      pink: '#a3144d',
      cyan: '#036a96',
      green: '#14710a',
      orange: '#a34d14',
      yellow: '#846e15',
      red: '#cb3a2a'
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
      base: pick('background'),
      surface0: pick('surface0'),
      surface1: pick('surface1'),
      surface2: pick('surface2'),
      overlay: pick('comment'),
      subtext: mix(pick('foreground'), pick('comment'), SUBTEXT_TOWARD_COMMENT),
      text: pick('foreground'),
      success: pick('green'),
      warning: pick('orange'),
      danger: pick('red'),
      info: pick('cyan')
    },
    accents: Object.fromEntries(ACCENTS.map((accent) => [accent, pick(accent)]))
  };
}

export const dracula: ThemeFamily = {
  id: 'dracula',
  name: 'Dracula',
  defaultAccent: 'purple',
  accents: accentOptions(ACCENTS),
  themes: VARIANTS.map(theme)
};
