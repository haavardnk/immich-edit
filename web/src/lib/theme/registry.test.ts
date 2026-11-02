import { describe, expect, it } from 'vitest';
import { backgroundColor, CONTRAST_PAIRS, pairContrast, pairName } from './contrast';
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
      for (const pair of CONTRAST_PAIRS) {
        expect(
          pairContrast(
            colors[pair.foreground],
            backgroundColor(colors, pair.background),
            pair.alpha
          ),
          `${theme.id} ${theme.accent} ${pairName(pair)}`
        ).toBeGreaterThanOrEqual(pair.minimum);
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
      FAMILIES.flatMap((family) =>
        family.themes.map((theme) => {
          const results = family.accents.map((accent) => resolveTheme(family, theme, accent.id));
          const palette = [
            ...new Set(
              results.flatMap((result) =>
                result.nudges.filter((nudge) => !nudge.startsWith('accent/'))
              )
            )
          ];
          const accents = results
            .filter((result) => result.nudges.some((nudge) => nudge.startsWith('accent/')))
            .map((result) => result.accent);
          return [theme.id, { palette, accents }] as const;
        })
      ).filter(([, { palette, accents }]) => palette.length > 0 || accents.length > 0)
    );
    expect(nudged).toMatchInlineSnapshot(`
      {
        "alucard": {
          "accents": [
            "cyan",
            "green",
            "orange",
            "yellow",
            "red",
          ],
          "palette": [
            "danger/mantle",
          ],
        },
        "catppuccin-frappe": {
          "accents": [
            "red",
          ],
          "palette": [
            "text@65/base",
          ],
        },
        "catppuccin-latte": {
          "accents": [
            "rosewater",
            "flamingo",
            "pink",
            "mauve",
            "red",
            "maroon",
            "peach",
            "yellow",
            "green",
            "teal",
            "sky",
            "sapphire",
            "blue",
            "lavender",
          ],
          "palette": [
            "text/surface1",
            "text@65/crust",
            "subtext/surface0",
            "success/base",
            "success/mantle",
            "warning/base",
            "warning/mantle",
            "danger/mantle",
            "info/base",
            "info/mantle",
            "overlay/base",
          ],
        },
        "dracula": {
          "accents": [
            "red",
          ],
          "palette": [],
        },
        "everforest-dark": {
          "accents": [
            "red",
          ],
          "palette": [
            "text@65/mantle",
            "text@65/base",
            "subtext/surface0",
          ],
        },
        "everforest-light": {
          "accents": [
            "green",
            "aqua",
            "blue",
            "purple",
            "red",
            "orange",
            "yellow",
          ],
          "palette": [
            "text/surface0",
            "text/surface1",
            "text@65/crust",
            "subtext/mantle",
            "subtext/surface0",
            "success/base",
            "success/mantle",
            "warning/base",
            "warning/mantle",
            "danger/base",
            "danger/mantle",
            "info/base",
            "info/mantle",
            "overlay/base",
          ],
        },
        "github-dark-dimmed": {
          "accents": [
            "purple",
            "pink",
            "red",
            "orange",
          ],
          "palette": [
            "text@65/crust",
            "text@65/base",
            "subtext/mantle",
            "subtext/base",
            "subtext/surface0",
            "danger/base",
            "overlay/base",
          ],
        },
        "github-light": {
          "accents": [
            "blue",
            "purple",
            "pink",
            "red",
            "orange",
            "yellow",
            "green",
          ],
          "palette": [
            "subtext/surface0",
          ],
        },
        "gruvbox-dark": {
          "accents": [
            "red",
          ],
          "palette": [
            "subtext/surface0",
            "danger/base",
          ],
        },
        "gruvbox-light": {
          "accents": [
            "green",
            "yellow",
            "aqua",
          ],
          "palette": [
            "text@65/crust",
            "subtext/mantle",
            "subtext/base",
            "subtext/surface0",
            "success/base",
            "warning/base",
            "overlay/base",
          ],
        },
        "immich-light": {
          "accents": [],
          "palette": [
            "text@65/crust",
            "success/base",
            "success/mantle",
            "warning/base",
            "warning/mantle",
            "danger/base",
            "danger/mantle",
            "info/base",
            "info/mantle",
            "overlay/base",
          ],
        },
        "nord": {
          "accents": [
            "frost-blue",
            "frost-deep",
            "red",
            "orange",
            "purple",
          ],
          "palette": [
            "danger/base",
          ],
        },
        "one-dark": {
          "accents": [
            "red",
          ],
          "palette": [
            "text@65/crust",
            "text@65/mantle",
            "text@65/base",
            "subtext/mantle",
            "subtext/base",
            "subtext/surface0",
            "danger/base",
            "overlay/base",
          ],
        },
        "one-light": {
          "accents": [
            "blue",
            "cyan",
            "green",
            "yellow",
            "orange",
            "red",
          ],
          "palette": [
            "text@65/crust",
            "subtext/surface0",
            "success/base",
            "success/mantle",
            "warning/base",
            "warning/mantle",
            "danger/base",
            "danger/mantle",
            "info/base",
            "info/mantle",
            "overlay/base",
          ],
        },
        "rose-pine": {
          "accents": [
            "pine",
          ],
          "palette": [
            "success/base",
          ],
        },
        "rose-pine-dawn": {
          "accents": [
            "love",
            "gold",
            "rose",
            "foam",
            "iris",
          ],
          "palette": [
            "text@65/crust",
            "subtext/mantle",
            "subtext/surface0",
            "warning/base",
            "warning/mantle",
            "danger/base",
            "danger/mantle",
            "info/base",
            "info/mantle",
            "overlay/base",
          ],
        },
        "rose-pine-moon": {
          "accents": [
            "pine",
          ],
          "palette": [
            "subtext/surface0",
            "success/base",
          ],
        },
        "solarized-dark": {
          "accents": [
            "yellow",
            "orange",
            "red",
            "magenta",
            "violet",
            "blue",
            "green",
          ],
          "palette": [
            "text/surface1",
            "text@65/crust",
            "text@65/mantle",
            "text@65/base",
            "subtext/surface0",
            "danger/base",
            "info/base",
            "overlay/base",
          ],
        },
        "solarized-light": {
          "accents": [
            "yellow",
            "orange",
            "red",
            "magenta",
            "violet",
            "blue",
            "cyan",
            "green",
          ],
          "palette": [
            "text/surface0",
            "text/surface1",
            "text@65/crust",
            "subtext/mantle",
            "subtext/surface0",
            "success/base",
            "success/mantle",
            "warning/base",
            "warning/mantle",
            "danger/base",
            "danger/mantle",
            "info/base",
            "info/mantle",
            "overlay/base",
          ],
        },
        "tokyo-night": {
          "accents": [],
          "palette": [
            "overlay/base",
          ],
        },
        "tokyo-night-day": {
          "accents": [
            "blue",
            "cyan",
            "magenta",
            "purple",
            "orange",
            "yellow",
            "green",
            "teal",
            "red",
          ],
          "palette": [
            "text/crust",
            "text/surface0",
            "text/surface1",
            "text@65/crust",
            "subtext/mantle",
            "subtext/surface0",
            "success/base",
            "success/mantle",
            "warning/base",
            "warning/mantle",
            "danger/base",
            "danger/mantle",
            "info/base",
            "info/mantle",
            "overlay/base",
          ],
        },
        "tokyo-night-moon": {
          "accents": [],
          "palette": [
            "subtext/surface0",
          ],
        },
        "tokyo-night-storm": {
          "accents": [
            "purple",
          ],
          "palette": [
            "overlay/base",
          ],
        },
      }
    `);
  });

  it('falls back to the default theme and accent for unknown ids', () => {
    const themes = resolveAppearance({
      mode: 'system',
      lightTheme: 'removed-theme',
      darkTheme: 'immich-edit-light',
      accents: { dark: 'removed-accent' }
    });
    expect(themes.light.id).toBe(DEFAULT_APPEARANCE.lightTheme);
    expect(themes.dark.id).toBe(DEFAULT_APPEARANCE.darkTheme);
    expect(themes.dark.accent).toBe('violet');
    expect(themeEntry('removed-theme', 'dark').theme.id).toBe('immich-edit-dark');
  });

  it('resolves the light and dark accents independently', () => {
    const themes = resolveAppearance({ ...DEFAULT_APPEARANCE, accents: { light: 'teal' } });
    expect(themes.light.accent).toBe('teal');
    expect(themes.dark.accent).toBe('violet');
  });

  it('gives explicit ramps eleven steps and only inherits dark stage colours', () => {
    for (const family of FAMILIES) {
      for (const theme of family.themes) {
        const ramps = [
          ...Object.values(theme.ramps?.primary ?? {}),
          ...(theme.ramps?.ladder ? [theme.ramps.ladder] : []),
          ...(theme.ramps?.gray ? [theme.ramps.gray] : [])
        ];
        for (const ramp of ramps) expect(ramp, theme.id).toHaveLength(11);
        if (theme.inherit?.includes('stage')) expect(theme.scheme, theme.id).toBe('dark');
      }
    }
  });

  it('ignores accent ids that name object properties', () => {
    const themes = resolveAppearance({
      ...DEFAULT_APPEARANCE,
      accents: { dark: 'constructor' }
    });
    expect(themes.dark.accent).toBe('violet');
  });

  it('lists themes by scheme', () => {
    expect(themesFor('light').every(({ theme }) => theme.scheme === 'light')).toBe(true);
    expect(themesFor('dark').map(({ theme }) => theme.id)).toContain('immich-edit-dark');
  });
});
