import {
  EXIF_LABELS,
  UNKNOWN_EXIF,
  exclusiveUnknown,
  exifSummary,
  type ExifFilterKey
} from './exifFilters';

export type ExifValues = Readonly<Record<ExifFilterKey, readonly string[]>>;

export interface ExifCrumb {
  level: number;
  label: string;
}

export const EXIF_PLURALS: Readonly<Record<ExifFilterKey, string>> = {
  make: 'Makes',
  model: 'Models',
  lensModel: 'Lenses',
  country: 'Countries',
  state: 'States',
  city: 'Cities'
};

function knownCount(values: readonly string[]): number {
  return values.filter((value) => value !== UNKNOWN_EXIF).length;
}

export function drillLevel(keys: readonly ExifFilterKey[], values: ExifValues): number {
  const last = keys.length - 1;
  const singles = keys.findIndex((key) => knownCount(values[key]) !== 1);
  const leading = singles === -1 ? keys.length : singles;
  const deepest = keys.findLastIndex((key) => values[key].length > 0);
  return Math.max(0, Math.min(last, Math.max(leading, deepest)));
}

export function drillCrumbs(
  keys: readonly ExifFilterKey[],
  values: ExifValues,
  level: number
): ExifCrumb[] {
  const [root] = keys;
  if (level === 0 || root === undefined) return [];
  return [
    { level: 0, label: EXIF_PLURALS[root] },
    ...keys.slice(0, level).map((key, index) => ({
      level: index + 1,
      label:
        values[key].length > 0
          ? exifSummary([...values[key]])
          : `Any ${EXIF_LABELS[key].toLowerCase()}`
    }))
  ];
}

export function pickExif(
  keys: readonly ExifFilterKey[],
  values: ExifValues,
  level: number,
  value: string,
  multiple: boolean
): Partial<Record<ExifFilterKey, string[]>> {
  const key = keys[level];
  if (key === undefined) return {};
  const current = values[key];
  const next = current.includes(value)
    ? current.filter((entry) => entry !== value)
    : multiple
      ? exclusiveUnknown([...current, value])
      : [value];
  return Object.fromEntries(
    keys.map((entry, index) => {
      if (index < level) return [entry, [...values[entry]]];
      return [entry, index === level ? next : []];
    })
  );
}
