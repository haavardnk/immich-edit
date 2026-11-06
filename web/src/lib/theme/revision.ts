import { themeCss } from './css';
import { faviconDataUrl } from './favicon';
import { resolveTheme } from './resolve';
import type { ThemeFamily } from './types';
import { fnv1a } from '#lib/utils/hash.ts';

export const THEME_REVISION_PLACEHOLDER = '%theme.revision%';

export function themeRevision(families: readonly ThemeFamily[]): string {
  const outputs = families.flatMap((family) =>
    family.themes.flatMap((theme) =>
      family.accents.map(({ id }) => {
        const resolved = resolveTheme(family, theme, id);
        return [
          themeCss([resolved]),
          resolved.palette.base,
          faviconDataUrl(resolved.accentColor)
        ].join('\n');
      })
    )
  );
  return fnv1a(outputs.join('\n'));
}
