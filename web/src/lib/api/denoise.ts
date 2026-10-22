import { getJson, sendJson, url } from './client';

export type DenoiseStatus =
  | { state: 'disabled' }
  | { state: 'model_missing' }
  | { state: 'idle' }
  | { state: 'queued' }
  | { state: 'running'; done: number; total: number }
  | { state: 'ready' }
  | { state: 'unsupported' }
  | { state: 'failed'; error: string };

export function getDenoiseStatus(assetId: string): Promise<DenoiseStatus> {
  return getJson<DenoiseStatus>(url`/api/assets/${assetId}/denoise`, undefined, { silent: true });
}

export function startDenoise(assetId: string): Promise<DenoiseStatus> {
  return sendJson<DenoiseStatus>(
    'POST',
    url`/api/assets/${assetId}/denoise`,
    undefined,
    undefined,
    {
      silent: true
    }
  );
}
