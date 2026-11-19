export const SAVED_VIEW_NAME_MAX = 80;

const SEPARATOR = ' · ';

export function savedViewName(values: readonly string[], fallback: string): string {
  const name = values.join(SEPARATOR) || fallback;
  return [...name].slice(0, SAVED_VIEW_NAME_MAX).join('').trim();
}

export function savedViewNameError(name: string): string | null {
  const size = [...name.trim()].length;
  if (size === 0) return 'Enter a name';
  return size > SAVED_VIEW_NAME_MAX ? `Up to ${SAVED_VIEW_NAME_MAX} characters` : null;
}
