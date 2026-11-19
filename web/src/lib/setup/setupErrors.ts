import { ApiError, isBackendDown } from '#lib/api/client.ts';

const BACKEND_DOWN = 'The immich-edit server is not responding. Check that it is running.';

const CLAIM_MESSAGES: Readonly<Record<string, string>> = {
  admin_required: 'That account is not an Immich administrator. Setup requires an admin.',
  invalid_credentials: 'Invalid credentials.',
  rate_limited: 'Too many attempts. Wait a moment and try again.',
  upstream_unavailable: 'Could not reach the Immich server. Check the URL.',
  conflict: 'This instance is already configured.',
  bad_request: 'The sign-in link expired. Start again.'
};

export const IMMICH_SILENT = "Immich didn't answer at this address.";

export function claimError(err: unknown, fallback: string): string {
  if (isBackendDown(err)) return BACKEND_DOWN;
  if (err instanceof ApiError) return CLAIM_MESSAGES[err.code] ?? (err.message || fallback);
  return err instanceof Error && err.message ? err.message : fallback;
}

export function probeError(err: unknown): string {
  if (isBackendDown(err)) return BACKEND_DOWN;
  if (err instanceof ApiError && err.message) return err.message;
  return 'Enter a valid Immich URL.';
}
