import { getJson, sendJson, url } from './client';

export interface SessionInfo {
  id: string;
  current: boolean;
  created_at: string;
  last_seen_at: string | null;
  user_agent: string | null;
  ip: string | null;
}

export async function listSessions(): Promise<SessionInfo[]> {
  const res = await getJson<{ sessions: SessionInfo[] }>('/api/auth/sessions');
  return res.sessions;
}

export async function revokeSession(id: string): Promise<void> {
  await sendJson<{ ok: boolean }>('DELETE', url`/api/auth/sessions/${id}`, undefined);
}

export async function revokeAllSessions(): Promise<void> {
  await sendJson<{ ok: boolean }>('POST', '/api/auth/sessions/revoke-all', {});
}
