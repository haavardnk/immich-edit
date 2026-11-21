export const SEARCH_FIELD_SIZES = {
  compact: '[&_input]:h-6 [&_input]:text-caption',
  toolbar:
    'h-7 [&_input]:h-7 [&_input]:text-caption touch:h-8 touch:[&_input]:h-11 touch:[&_input]:text-sm',
  default: '[&_input]:h-7.5 [&_input]:text-label touch:[&_input]:h-11 touch:[&_input]:text-body'
} as const;

export type SearchFieldSize = keyof typeof SEARCH_FIELD_SIZES;
