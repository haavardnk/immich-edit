import type { ClientInit, HandleClientError } from '@sveltejs/kit/hooks';
import { installClientLog, reportClientError } from '#lib/api/clientLog.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { errorMessage } from '#lib/utils/errors.ts';

export const init: ClientInit = () => {
  installClientLog();
};

export const handleError: HandleClientError = ({ kind, error }) => {
  if (kind !== 'unknown') return;
  const message = errorMessage(error);
  toasts.push('error', message, 12000);
  reportClientError('svelte', error);
  return { message };
};
