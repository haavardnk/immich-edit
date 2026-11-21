import { charCount } from '#lib/utils/charCount.ts';

export const DISPLAY_NAME_MAX = 80;

export function displayNameTooLong(name: string): boolean {
  return charCount(name.trim()) > DISPLAY_NAME_MAX;
}
