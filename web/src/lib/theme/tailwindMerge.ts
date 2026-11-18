import { extendTailwindMerge } from 'tailwind-merge';

export * from 'tailwind-merge';

export const TYPE_SCALE = ['nano', 'micro', 'caption', 'label', 'body', 'title'] as const;

export const twMerge = extendTailwindMerge({ extend: { theme: { text: [...TYPE_SCALE] } } });
