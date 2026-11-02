import { mix, shiftLightness } from '#lib/theme/color.ts';
import type { ThemeFamily } from '#lib/theme/types.ts';

const NORD = {
  nord0: '#2e3440',
  nord1: '#3b4252',
  nord2: '#434c5e',
  nord3: '#4c566a',
  nord4: '#d8dee9',
  nord6: '#eceff4',
  nord7: '#8fbcbb',
  nord8: '#88c0d0',
  nord9: '#81a1c1',
  nord10: '#5e81ac',
  nord11: '#bf616a',
  nord12: '#d08770',
  nord13: '#ebcb8b',
  nord14: '#a3be8c',
  nord15: '#b48ead'
} as const;

const ACCENTS = [
  { id: 'frost-teal', name: 'Frost teal', color: NORD.nord7 },
  { id: 'frost-cyan', name: 'Frost cyan', color: NORD.nord8 },
  { id: 'frost-blue', name: 'Frost blue', color: NORD.nord9 },
  { id: 'frost-deep', name: 'Frost deep blue', color: NORD.nord10 },
  { id: 'red', name: 'Red', color: NORD.nord11 },
  { id: 'orange', name: 'Orange', color: NORD.nord12 },
  { id: 'yellow', name: 'Yellow', color: NORD.nord13 },
  { id: 'green', name: 'Green', color: NORD.nord14 },
  { id: 'purple', name: 'Purple', color: NORD.nord15 }
] as const;

export const nord: ThemeFamily = {
  id: 'nord',
  name: 'Nord',
  defaultAccent: 'frost-cyan',
  accents: ACCENTS.map(({ id, name }) => ({ id, name })),
  themes: [
    {
      id: 'nord',
      name: 'Nord',
      scheme: 'dark',
      palette: {
        crust: shiftLightness(NORD.nord0, -0.05),
        mantle: shiftLightness(NORD.nord0, -0.025),
        base: NORD.nord0,
        surface0: NORD.nord1,
        surface1: NORD.nord2,
        surface2: NORD.nord3,
        overlay: mix(NORD.nord3, NORD.nord4, 0.35),
        subtext: NORD.nord4,
        text: NORD.nord6,
        success: NORD.nord14,
        warning: NORD.nord13,
        danger: NORD.nord11,
        info: NORD.nord9
      },
      accents: Object.fromEntries(ACCENTS.map(({ id, color }) => [id, color]))
    }
  ]
};
