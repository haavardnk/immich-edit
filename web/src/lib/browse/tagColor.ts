const HEX_COLOR = /^#?(?:[0-9a-f]{3}|[0-9a-f]{4}|[0-9a-f]{6}|[0-9a-f]{8})$/i;

export function cssTagColor(raw: string | null | undefined): string | null {
  const value = raw?.trim();
  if (!value || !HEX_COLOR.test(value)) return null;
  return value.startsWith('#') ? value : `#${value}`;
}
