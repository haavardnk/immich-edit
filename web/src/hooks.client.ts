import type { ClientInit, HandleClientError } from '@sveltejs/kit';
import { installClientLog, reportClientError } from '$lib/api/clientLog';
import { toasts } from '$lib/stores/toasts.svelte';
import { errorMessage } from '$lib/utils/errors';

export const init: ClientInit = () => {
  installClientLog();
};

export const handleError: HandleClientError = ({ error, status }) => {
  const message = errorMessage(error);
  if (status !== 404) {
    toasts.push('error', message, 12000);
    reportClientError('svelte', error);
  }
  return { message };
};
