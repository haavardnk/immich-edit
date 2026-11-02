import type { AccentOption } from '#lib/theme/types.ts';

export function accentOptions(ids: readonly string[]): AccentOption[] {
  return ids.map((id) => ({
    id,
    name: `${id.charAt(0).toUpperCase()}${id.slice(1).replaceAll('-', ' ')}`
  }));
}
