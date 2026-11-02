import type { Ramp, ThemeDefinition, ThemeFamily } from '#lib/theme/types.ts';

const PLUM: Ramp = [
  '#f8f7f9',
  '#ebe8ee',
  '#d5d0d9',
  '#b7b0bd',
  '#93899b',
  '#6e6275',
  '#504459',
  '#392f41',
  '#29212f',
  '#1c171f',
  '#120f15'
];

const VIOLET_DARK: Ramp = [
  '#221533',
  '#2f1d46',
  '#3c2559',
  '#4d306f',
  '#7046a0',
  '#b7a1fb',
  '#c4b5fd',
  '#d0c5fd',
  '#ddd6fe',
  '#ede9fe',
  '#f5f3ff'
];

const dark: ThemeDefinition = {
  id: 'immich-edit-dark',
  name: 'immich-edit Dark',
  scheme: 'dark',
  palette: {
    crust: '#0b0b0d',
    mantle: '#0d0d0f',
    base: '#000000',
    surface0: '#171717',
    surface1: '#262626',
    surface2: '#404040',
    overlay: '#d4d4d4',
    subtext: '#a3a3a3',
    text: '#dbdbdb',
    success: '#48ed98',
    warning: '#ffd198',
    danger: '#f67d7d',
    info: '#7ab7ff'
  },
  accents: {
    violet: '#b7a1fb',
    indigo: '#9babff',
    blue: '#74b6ff',
    sky: '#43c1f2',
    teal: '#1bcbc4',
    amber: '#e1a54a',
    coral: '#f69472',
    rose: '#f68ea7'
  },
  inherit: ['semantic', 'ladder', 'surface', 'stage'],
  ramps: { primary: { violet: VIOLET_DARK }, gray: PLUM }
};

const light: ThemeDefinition = {
  id: 'immich-edit-light',
  name: 'immich-edit Light',
  scheme: 'light',
  palette: {
    crust: '#f1eff3',
    mantle: '#f8f7f9',
    base: '#ffffff',
    surface0: '#ebe8ee',
    surface1: '#d5d0d9',
    surface2: '#b7b0bd',
    overlay: '#93899b',
    subtext: '#504459',
    text: '#1c171f',
    success: '#15803d',
    warning: '#a16207',
    danger: '#dc2626',
    info: '#2563eb'
  },
  accents: {
    violet: '#7159b0',
    indigo: '#5763b0',
    blue: '#296baf',
    sky: '#007094',
    teal: '#00756f',
    amber: '#8d5f00',
    coral: '#ae4823',
    rose: '#ad4363'
  },
  ramps: { ladder: PLUM, gray: PLUM }
};

export const immichEdit: ThemeFamily = {
  id: 'immich-edit',
  name: 'immich-edit',
  defaultAccent: 'violet',
  accents: [
    { id: 'violet', name: 'Violet' },
    { id: 'indigo', name: 'Indigo' },
    { id: 'blue', name: 'Blue' },
    { id: 'sky', name: 'Sky' },
    { id: 'teal', name: 'Teal' },
    { id: 'amber', name: 'Amber' },
    { id: 'coral', name: 'Coral' },
    { id: 'rose', name: 'Rose' }
  ],
  themes: [dark, light]
};
