import type { HandleClientError } from '@sveltejs/kit';
import { toasts } from '$lib/stores/toasts.svelte';
import { errorMessage } from '$lib/utils/errors';

export const handleError: HandleClientError = ({ error, status }) => {
  const message = errorMessage(error);
  if (status !== 404) toasts.push('error', message, 12000);
  return { message };
};
