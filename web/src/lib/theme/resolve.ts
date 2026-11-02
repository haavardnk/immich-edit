import { gate } from './contrast';
import { themeTokens } from './tokens';
import type { ResolvedTheme, ThemeDefinition, ThemeFamily } from './types';

export function resolveTheme(
  family: ThemeFamily,
  theme: ThemeDefinition,
  requestedAccent: string | undefined
): ResolvedTheme {
  const accent =
    requestedAccent !== undefined && Object.hasOwn(theme.accents, requestedAccent)
      ? requestedAccent
      : family.defaultAccent;
  const gated = gate(theme.palette, theme.accents[accent] ?? theme.palette.text);
  return {
    id: theme.id,
    family: family.id,
    scheme: theme.scheme,
    accent,
    palette: gated.palette,
    accentColor: gated.accent,
    tokens: themeTokens(theme, gated.palette, accent, gated.accent),
    nudges: gated.nudges
  };
}
