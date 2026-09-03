import { getJson, sendJson, url } from './client';

export interface AdminUser {
  id: string;
  email: string;
  name: string;
  is_admin: boolean;
  access_enabled: boolean;
}

export interface InstanceInfo {
  server_epoch: number;
  immich_url: string;
  configured_at: string | null;
}

export interface RebindBody {
  immich_url: string;
  confirm_hostname: string;
  email?: string;
  password?: string;
  api_key?: string;
}

export async function listUsers(): Promise<AdminUser[]> {
  const res = await getJson<{ users: AdminUser[] }>('/api/admin/users');
  return res.users;
}

export async function setUserAccess(id: string, enabled: boolean): Promise<void> {
  await sendJson<{ ok: boolean }>('PUT', url`/api/admin/users/${id}/access`, { enabled });
}

export async function purgeUserData(id: string): Promise<void> {
  await sendJson<{ ok: boolean }>('DELETE', url`/api/admin/users/${id}/data`, undefined);
}

export async function getInstance(): Promise<InstanceInfo> {
  return getJson<InstanceInfo>('/api/admin/instance');
}

export async function rebindInstance(body: RebindBody): Promise<void> {
  await sendJson<{ ok: boolean }>('POST', '/api/admin/instance/rebind', body);
}
