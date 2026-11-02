import type { ThemeFamily } from '#lib/theme/types.ts';

export const immichClassic: ThemeFamily = {
  id: 'immich',
  name: 'Immich',
  defaultAccent: 'blue',
  accents: [{ id: 'blue', name: 'Blue' }],
  themes: [
    {
      id: 'immich-light',
      name: 'Immich Light',
      scheme: 'light',
      palette: {
        crust: '#f5f5f5',
        mantle: '#f5f5f5',
        base: '#ffffff',
        surface0: '#f5f5f5',
        surface1: '#e5e5e5',
        surface2: '#d4d4d4',
        overlay: '#a1a1a1',
        subtext: '#525252',
        text: '#3d3d3d',
        success: '#10c14d',
        warning: '#ffb003',
        danger: '#fa2921',
        info: '#1984e9'
      },
      accents: { blue: '#4250af' },
      inherit: ['primary', 'ladder', 'gray']
    },
    {
      id: 'immich-dark',
      name: 'Immich Dark',
      scheme: 'dark',
      palette: {
        crust: '#0b0b0d',
        mantle: '#101116',
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
      accents: { blue: '#accbfa' },
      inherit: ['primary', 'semantic', 'ladder', 'surface', 'gray', 'stage']
    }
  ]
};
