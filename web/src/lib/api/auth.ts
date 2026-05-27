import { sendJson } from './client';

export async function login(token: string): Promise<void> {
  await sendJson<{ ok: boolean }>('POST', '/api/auth/login', { token });
}

export async function logout(): Promise<void> {
  await sendJson<{ ok: boolean }>('POST', '/api/auth/logout', {});
}
