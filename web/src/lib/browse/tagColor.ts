const HEX_COLOR = /^#?(?:[0-9a-f]{3}|[0-9a-f]{4}|[0-9a-f]{6}|[0-9a-f]{8})$/i;

export function cssTagColor(raw: string | null | undefined): string | null {
  const value = raw?.trim();
  if (!value || !HEX_COLOR.test(value)) return null;
  return value.startsWith('#') ? value : `#${value}`;
}

export interface TagSwatch {
  value: string;
  label: string;
}

export const TAG_SWATCHES: readonly TagSwatch[] = [
  { value: '#e5484d', label: 'Red' },
  { value: '#f76b15', label: 'Orange' },
  { value: '#f5c542', label: 'Yellow' },
  { value: '#46a758', label: 'Green' },
  { value: '#12a594', label: 'Teal' },
  { value: '#3e8ed0', label: 'Blue' },
  { value: '#8e4ec6', label: 'Purple' },
  { value: '#d6409f', label: 'Pink' }
];

export function sameTagColor(a: string | null | undefined, b: string | null | undefined): boolean {
  return (cssTagColor(a)?.toLowerCase() ?? null) === (cssTagColor(b)?.toLowerCase() ?? null);
}

export function tagSwatches(current: string | null | undefined): TagSwatch[] {
  const custom = cssTagColor(current);
  if (!custom || TAG_SWATCHES.some((swatch) => sameTagColor(swatch.value, custom))) {
    return [...TAG_SWATCHES];
  }
  return [...TAG_SWATCHES, { value: custom, label: 'Color from Immich' }];
}
