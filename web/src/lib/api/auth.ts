import { getJson, sendJson } from './client';

export type AuthKind = 'password' | 'apikey';

export interface SessionUser {
  id: string;
  email: string;
  name: string;
  is_admin: boolean;
  auth_kind: AuthKind;
}

export interface SetupStatus {
  configured: boolean;
}

export interface SetupBody {
  immich_url: string;
  email?: string;
  password?: string;
  api_key?: string;
}

export async function setupStatus(): Promise<SetupStatus> {
  return getJson<SetupStatus>('/api/setup/status', undefined, { silent: true });
}

export async function completeSetup(body: SetupBody): Promise<SessionUser> {
  return sendJson<SessionUser>('POST', '/api/setup/complete', body, undefined, { silent: true });
}

export async function loginPassword(email: string, password: string): Promise<SessionUser> {
  return sendJson<SessionUser>(
    'POST',
    '/api/auth/login/password',
    { email, password },
    undefined,
    { silent: true }
  );
}

export async function loginApiKey(apiKey: string): Promise<SessionUser> {
  return sendJson<SessionUser>('POST', '/api/auth/login/api-key', { api_key: apiKey }, undefined, {
    silent: true
  });
}

export async function me(): Promise<SessionUser> {
  return getJson<SessionUser>('/api/auth/me', undefined, { silent: true });
}

export async function logout(): Promise<void> {
  await sendJson<{ ok: boolean }>('POST', '/api/auth/logout', {}, undefined, { silent: true });
}
