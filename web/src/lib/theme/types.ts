export type Scheme = 'light' | 'dark';

export const THEME_MODES = ['system', 'light', 'dark'] as const;

export type ThemeMode = (typeof THEME_MODES)[number];

export type SurfaceSlot =
  | 'crust'
  | 'mantle'
  | 'base'
  | 'surface0'
  | 'surface1'
  | 'surface2'
  | 'overlay'
  | 'subtext'
  | 'text';

export const SEMANTIC_SLOTS = ['success', 'warning', 'danger', 'info'] as const;

export type SemanticSlot = (typeof SEMANTIC_SLOTS)[number];

export type PaletteSlot = SurfaceSlot | SemanticSlot;

export type Palette = Readonly<Record<PaletteSlot, string>>;

export type Ramp = readonly string[];

export const TOKEN_GROUPS = ['primary', 'semantic', 'ladder', 'surface', 'gray', 'stage'] as const;

export type TokenGroup = (typeof TOKEN_GROUPS)[number];

interface ThemeRamps {
  primary?: Readonly<Record<string, Ramp>>;
  ladder?: Ramp;
  gray?: Ramp;
}

export interface ThemeDefinition {
  id: string;
  name: string;
  scheme: Scheme;
  palette: Palette;
  accents: Readonly<Record<string, string>>;
  inherit?: readonly TokenGroup[];
  ramps?: ThemeRamps;
}

export interface AccentOption {
  id: string;
  name: string;
}

export interface ThemeFamily {
  id: string;
  name: string;
  defaultAccent: string;
  accents: readonly AccentOption[];
  themes: readonly ThemeDefinition[];
}

export interface Appearance {
  mode: ThemeMode;
  lightTheme: string;
  darkTheme: string;
  accents: Readonly<Record<string, string>>;
}

export interface ResolvedTheme {
  id: string;
  family: string;
  scheme: Scheme;
  accent: string;
  palette: Palette;
  accentColor: string;
  tokens: Readonly<Record<string, string>>;
  nudges: readonly string[];
}
