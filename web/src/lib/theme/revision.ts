import { themeCss } from './css';
import { faviconDataUrl } from './favicon';
import { resolveTheme } from './resolve';
import type { ThemeFamily } from './types';

export const THEME_REVISION_PLACEHOLDER = '%theme.revision%';

const FNV_OFFSET = 0x811c9dc5;
const FNV_PRIME = 0x01000193;

function fnv1a(text: string): string {
  let hash = FNV_OFFSET;
  for (let index = 0; index < text.length; index++) {
    hash ^= text.charCodeAt(index);
    hash = Math.imul(hash, FNV_PRIME);
  }
  return (hash >>> 0).toString(16).padStart(8, '0');
}

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
