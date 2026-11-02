import { accentRamp, neutralLadder, RAMP_STEPS } from './ramps';
import {
  SEMANTIC_SLOTS,
  TOKEN_GROUPS,
  type Palette,
  type Ramp,
  type ThemeDefinition,
  type TokenGroup
} from './types';

type Tokens = Record<string, string>;

function rampTokens(prefix: string, ramp: Ramp): Tokens {
  return Object.fromEntries(
    RAMP_STEPS.map((step, index) => [`${prefix}-${step}`, ramp[index] ?? ''])
  );
}

function primaryTokens(
  theme: ThemeDefinition,
  palette: Palette,
  accent: string,
  accentColor: string
): Tokens {
  const ramp =
    theme.ramps?.primary?.[accent] ??
    accentRamp(accentColor, palette.base, palette.text, theme.scheme);
  return rampTokens('--immich-ui-primary', ramp);
}

function semanticTokens(theme: ThemeDefinition, palette: Palette): Tokens {
  return Object.assign(
    {},
    ...SEMANTIC_SLOTS.map((slot) =>
      rampTokens(
        `--immich-ui-${slot}`,
        accentRamp(palette[slot], palette.base, palette.text, theme.scheme)
      )
    )
  );
}

function ladderOf(theme: ThemeDefinition, palette: Palette): Ramp {
  return theme.ramps?.ladder ?? neutralLadder(palette);
}

function ladderTokens(theme: ThemeDefinition, palette: Palette): Tokens {
  const ladder = ladderOf(theme, palette);
  return {
    ...rampTokens('--immich-ui-light', ladder),
    ...rampTokens('--immich-ui-dark', [...ladder].reverse())
  };
}

function surfaceTokens(theme: ThemeDefinition, palette: Palette): Tokens {
  return {
    '--immich-ui-light': palette.base,
    '--immich-ui-dark': palette.text,
    '--immich-ui-muted': palette.overlay,
    '--immich-ui-gray': palette.mantle,
    '--immich-ui-default-border': `var(--immich-ui-light-${theme.scheme === 'dark' ? 200 : 300})`
  };
}

function grayTokens(theme: ThemeDefinition, palette: Palette): Tokens {
  const ladder = ladderOf(theme, palette);
  const gray = theme.ramps?.gray ?? (theme.scheme === 'dark' ? [...ladder].reverse() : ladder);
  return rampTokens('--color-gray', gray);
}

function stageTokens(palette: Palette): Tokens {
  return {
    '--color-editor-chrome': palette.crust,
    '--color-editor-panel': palette.mantle,
    '--color-auth-stage': palette.crust
  };
}

export function themeTokens(
  theme: ThemeDefinition,
  palette: Palette,
  accent: string,
  accentColor: string
): Readonly<Tokens> {
  const groups: Record<TokenGroup, () => Tokens> = {
    primary: () => primaryTokens(theme, palette, accent, accentColor),
    semantic: () => semanticTokens(theme, palette),
    ladder: () => ladderTokens(theme, palette),
    surface: () => surfaceTokens(theme, palette),
    gray: () => grayTokens(theme, palette),
    stage: () => stageTokens(palette)
  };
  const inherited = new Set(theme.inherit ?? []);
  return Object.assign(
    {},
    ...TOKEN_GROUPS.filter((group) => !inherited.has(group)).map((group) => groups[group]())
  );
}
