import { isRecord } from '#lib/utils/isRecord.ts';
import { THEME_MODES, type Appearance } from './types';

function isStringRecord(value: unknown): value is Record<string, string> {
  return isRecord(value) && Object.values(value).every((entry) => typeof entry === 'string');
}

export function isAppearance(value: unknown): value is Appearance {
  return (
    isRecord(value) &&
    THEME_MODES.some((mode) => mode === value.mode) &&
    typeof value.lightTheme === 'string' &&
    typeof value.darkTheme === 'string' &&
    isStringRecord(value.accents)
  );
}

export function sameAppearance(a: Appearance, b: Appearance): boolean {
  const accents = (value: Appearance): string =>
    JSON.stringify(Object.entries(value.accents).sort(([x], [y]) => x.localeCompare(y)));
  return (
    a.mode === b.mode &&
    a.lightTheme === b.lightTheme &&
    a.darkTheme === b.darkTheme &&
    accents(a) === accents(b)
  );
}
