import { mix, shiftLightness } from '#lib/theme/color.ts';
import { accentOptions } from '#lib/theme/families/accentOptions.ts';
import type { Scheme, ThemeDefinition, ThemeFamily } from '#lib/theme/types.ts';

const ACCENTS = ['yellow', 'orange', 'red', 'magenta', 'violet', 'blue', 'cyan', 'green'] as const;

const COLORS = {
  base03: '#002b36',
  base02: '#073642',
  base01: '#586e75',
  base00: '#657b83',
  base0: '#839496',
  base1: '#93a1a1',
  base2: '#eee8d5',
  base3: '#fdf6e3',
  yellow: '#b58900',
  orange: '#cb4b16',
  red: '#dc322f',
  magenta: '#d33682',
  violet: '#6c71c4',
  blue: '#268bd2',
  cyan: '#2aa198',
  green: '#859900'
} as const;

interface Roles {
  base: string;
  raised: string;
  overlay: string;
  subtext: string;
  text: string;
}

const SHIFT: Readonly<Record<Scheme, { mantle: number; crust: number }>> = {
  dark: { mantle: -0.025, crust: -0.05 },
  light: { mantle: -0.015, crust: -0.03 }
};

function theme(id: string, name: string, scheme: Scheme, roles: Roles): ThemeDefinition {
  return {
    id,
    name,
    scheme,
    palette: {
      crust: shiftLightness(roles.base, SHIFT[scheme].crust),
      mantle: shiftLightness(roles.base, SHIFT[scheme].mantle),
      base: roles.base,
      surface0: roles.raised,
      surface1: mix(roles.raised, roles.overlay, 0.25),
      surface2: mix(roles.raised, roles.overlay, 0.5),
      overlay: roles.overlay,
      subtext: roles.subtext,
      text: roles.text,
      success: COLORS.green,
      warning: COLORS.yellow,
      danger: COLORS.red,
      info: COLORS.blue
    },
    accents: Object.fromEntries(ACCENTS.map((accent) => [accent, COLORS[accent]]))
  };
}

export const solarized: ThemeFamily = {
  id: 'solarized',
  name: 'Solarized',
  defaultAccent: 'blue',
  accents: accentOptions(ACCENTS),
  themes: [
    theme('solarized-dark', 'Solarized Dark', 'dark', {
      base: COLORS.base03,
      raised: COLORS.base02,
      overlay: COLORS.base01,
      subtext: COLORS.base0,
      text: COLORS.base1
    }),
    theme('solarized-light', 'Solarized Light', 'light', {
      base: COLORS.base3,
      raised: COLORS.base2,
      overlay: COLORS.base1,
      subtext: COLORS.base00,
      text: COLORS.base01
    })
  ]
};
