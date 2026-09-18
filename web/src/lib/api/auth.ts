import { getJson, sendJson } from './client';

export type AuthKind = 'password' | 'apikey' | 'oauth';

export interface SessionUser {
  id: string;
  email: string;
  name: string;
  is_admin: boolean;
  auth_kind: AuthKind;
  next?: string;
}

export interface AuthProviders {
  oauth: boolean;
  password_login: boolean;
  auto_launch: boolean;
  button_text: string;
  degraded: boolean;
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
  return sendJson<SessionUser>('POST', '/api/auth/login/password', { email, password }, undefined, {
    silent: true
  });
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

export async function authProviders(): Promise<AuthProviders> {
  return getJson<AuthProviders>('/api/auth/providers', undefined, { silent: true });
}

export async function startOAuthLogin(redirectUri: string, next?: string): Promise<string> {
  const body = await sendJson<{ url: string }>(
    'POST',
    '/api/auth/oauth/start',
    { redirect_uri: redirectUri, next },
    undefined,
    { silent: true }
  );
  return body.url;
}

export async function completeOAuthLogin(url: string): Promise<SessionUser> {
  return sendJson<SessionUser>('POST', '/api/auth/oauth/callback', { url }, undefined, {
    silent: true
  });
}

export async function setupProviders(immichUrl: string): Promise<AuthProviders> {
  const query = new URLSearchParams({ immich_url: immichUrl });
  return getJson<AuthProviders>(`/api/setup/providers?${query}`, undefined, { silent: true });
}

export async function startOAuthSetup(immichUrl: string, redirectUri: string): Promise<string> {
  const body = await sendJson<{ url: string }>(
    'POST',
    '/api/setup/oauth/start',
    { immich_url: immichUrl, redirect_uri: redirectUri },
    undefined,
    { silent: true }
  );
  return body.url;
}

export async function completeOAuthSetup(url: string): Promise<SessionUser> {
  return sendJson<SessionUser>('POST', '/api/setup/oauth/complete', { url }, undefined, {
    silent: true
  });
}
