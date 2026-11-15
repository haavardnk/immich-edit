import { toasts } from '#lib/stores/toasts.svelte.ts';

export async function copyPath(path: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(path);
    toasts.push('success', 'Copied the file path');
  } catch (error) {
    toasts.fail('Could not copy the path', error);
  }
}
