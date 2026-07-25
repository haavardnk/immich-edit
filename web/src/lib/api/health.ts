import { getJson } from './client';

export async function live(): Promise<void> {
  await getJson<unknown>('/api/health/live', undefined, { silent: true });
}
