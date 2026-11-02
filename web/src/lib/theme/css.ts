import type { ResolvedTheme } from './types';

export function themeCss(themes: readonly ResolvedTheme[]): string {
  return themes
    .map((theme) => {
      const declarations = Object.entries(theme.tokens)
        .map(([name, value]) => `${name}:${value};`)
        .join('');
      return `:root.${theme.scheme}{${declarations}}`;
    })
    .join('');
}
