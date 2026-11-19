import {
  listSessions,
  revokeAllSessions,
  revokeSession,
  type SessionInfo
} from '#lib/api/account.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';

class SignedInDevicesStore {
  sessions = $state<SessionInfo[]>([]);
  loading = $state(false);
  loaded = $state(false);

  readonly count = $derived(this.loaded ? this.sessions.length : null);

  load = async (): Promise<void> => {
    if (this.loading) return;
    this.loading = true;
    try {
      this.sessions = await listSessions();
      this.loaded = true;
    } catch {
      return;
    } finally {
      this.loading = false;
    }
  };

  revoke = async (id: string): Promise<void> => {
    try {
      await revokeSession(id);
    } catch (e) {
      toasts.fail('Failed to revoke session', e);
    }
    await this.load();
  };

  revokeOthers = async (): Promise<void> => {
    try {
      await revokeAllSessions();
    } catch (e) {
      toasts.fail('Failed to revoke sessions', e);
    }
    await this.load();
  };
}

export const signedInDevices = new SignedInDevicesStore();
