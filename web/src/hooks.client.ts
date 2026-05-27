import type { HandleClientError } from '@sveltejs/kit';
import { toasts } from '$lib/stores/toasts.svelte';

export const handleError: HandleClientError = ({ error, status }) => {
  const message = error instanceof Error ? error.message : String(error);
  if (status !== 404) toasts.push('error', message, 12000);
  return { message };
};
