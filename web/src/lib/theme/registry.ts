import { immichEdit } from './families/immichEdit';
import { resolveTheme } from './resolve';
import type { Appearance, ResolvedTheme, Scheme, ThemeDefinition, ThemeFamily } from './types';

export interface ThemeEntry {
  family: ThemeFamily;
  theme: ThemeDefinition;
}

export const FAMILIES: readonly ThemeFamily[] = [immichEdit];

export const DEFAULT_THEME: Readonly<Record<Scheme, string>> = {
  light: 'immich-edit-light',
  dark: 'immich-edit-dark'
};

export const DEFAULT_APPEARANCE: Appearance = {
  mode: 'dark',
  lightTheme: DEFAULT_THEME.light,
  darkTheme: DEFAULT_THEME.dark,
  accents: {}
};

const ENTRIES: readonly ThemeEntry[] = FAMILIES.flatMap((family) =>
  family.themes.map((theme) => ({ family, theme }))
);

export function themesFor(scheme: Scheme): ThemeEntry[] {
  return ENTRIES.filter(({ theme }) => theme.scheme === scheme);
}

export function themeEntry(id: string, scheme: Scheme): ThemeEntry {
  const match = (candidate: string): ThemeEntry | undefined =>
    ENTRIES.find(({ theme }) => theme.id === candidate && theme.scheme === scheme);
  const found = match(id) ?? match(DEFAULT_THEME[scheme]);
  if (!found) throw new Error(`no ${scheme} theme registered as ${DEFAULT_THEME[scheme]}`);
  return found;
}

export function resolveAppearance(appearance: Appearance): Record<Scheme, ResolvedTheme> {
  const resolve = (id: string, scheme: Scheme): ResolvedTheme => {
    const { family, theme } = themeEntry(id, scheme);
    return resolveTheme(family, theme, appearance.accents[scheme]);
  };
  return {
    light: resolve(appearance.lightTheme, 'light'),
    dark: resolve(appearance.darkTheme, 'dark')
  };
}
