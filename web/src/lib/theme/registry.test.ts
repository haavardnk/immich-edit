import { describe, expect, it } from 'vitest';
import { contrastRatio } from './color';
import { backgroundColor, CONTRAST_PAIRS } from './contrast';
import { DEFAULT_APPEARANCE, FAMILIES, resolveAppearance, themeEntry, themesFor } from './registry';
import { resolveTheme } from './resolve';
import type { ResolvedTheme, TokenGroup } from './types';

const ID = /^[a-z0-9-]{1,64}$/;

const ROLE_GROUP: Readonly<Record<string, TokenGroup>> = {
  accent: 'primary',
  success: 'semantic',
  warning: 'semantic',
  danger: 'semantic',
  info: 'semantic',
  text: 'surface',
  overlay: 'surface',
  subtext: 'ladder'
};

const resolved: ResolvedTheme[] = FAMILIES.flatMap((family) =>
  family.themes.flatMap((theme) =>
    family.accents.map((accent) => resolveTheme(family, theme, accent.id))
  )
);

describe('theme registry', () => {
  it('uses unique ids the backend accepts', () => {
    const themeIds = FAMILIES.flatMap((family) => family.themes.map((theme) => theme.id));
    expect(new Set(themeIds).size).toBe(themeIds.length);
    for (const family of FAMILIES) {
      expect(family.id).toMatch(ID);
      for (const accent of family.accents) expect(accent.id).toMatch(ID);
    }
    for (const id of themeIds) expect(id).toMatch(ID);
  });

  it('gives every theme an anchor for each family accent', () => {
    for (const family of FAMILIES) {
      expect(family.accents.map((accent) => accent.id)).toContain(family.defaultAccent);
      for (const theme of family.themes) {
        expect(Object.keys(theme.accents).sort()).toEqual(
          family.accents.map((accent) => accent.id).sort()
        );
      }
    }
  });

  it('passes every contrast pair for every theme and accent', () => {
    for (const theme of resolved) {
      const colors = { ...theme.palette, accent: theme.accentColor };
      for (const { foreground, background, minimum } of CONTRAST_PAIRS) {
        expect(
          contrastRatio(colors[foreground], backgroundColor(colors, background)),
          `${theme.id} ${theme.accent} ${foreground}/${background}`
        ).toBeGreaterThanOrEqual(minimum);
      }
    }
  });

  it('never needs a nudge in a group the theme inherits', () => {
    for (const family of FAMILIES) {
      for (const theme of family.themes) {
        const inherited = new Set(theme.inherit ?? []);
        for (const accent of family.accents) {
          const { nudges } = resolveTheme(family, theme, accent.id);
          const explicitAccent = theme.ramps?.primary?.[accent.id] !== undefined;
          const hidden = nudges.filter((nudge) => {
            const role = nudge.split('/')[0] ?? '';
            const group = ROLE_GROUP[role];
            if (role === 'accent' && explicitAccent) return true;
            return group !== undefined && inherited.has(group);
          });
          expect(hidden, `${theme.id} ${accent.id}`).toEqual([]);
        }
      }
    }
  });

  it('records the nudges every built-in needs', () => {
    const nudged = Object.fromEntries(
      resolved
        .filter((theme) => theme.nudges.length > 0)
        .map((theme) => [`${theme.id}/${theme.accent}`, theme.nudges])
    );
    expect(nudged).toEqual({});
  });

  it('falls back to the default theme and accent for unknown ids', () => {
    const themes = resolveAppearance({
      mode: 'system',
      lightTheme: 'removed-theme',
      darkTheme: 'immich-edit-light',
      accents: { 'immich-edit': 'removed-accent' }
    });
    expect(themes.light.id).toBe(DEFAULT_APPEARANCE.lightTheme);
    expect(themes.dark.id).toBe(DEFAULT_APPEARANCE.darkTheme);
    expect(themes.dark.accent).toBe('violet');
    expect(themeEntry('removed-theme', 'dark').theme.id).toBe('immich-edit-dark');
  });

  it('lists themes by scheme', () => {
    expect(themesFor('light').every(({ theme }) => theme.scheme === 'light')).toBe(true);
    expect(themesFor('dark').map(({ theme }) => theme.id)).toContain('immich-edit-dark');
  });
});
