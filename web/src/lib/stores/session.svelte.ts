import type { SessionUser } from '$lib/api/auth';
import { me } from '$lib/api/auth';
import { ApiError } from '$lib/api/client';

class SessionStore {
  user = $state<SessionUser | null>(null);

  get isAdmin(): boolean {
    return this.user?.is_admin ?? false;
  }

  set = (user: SessionUser): void => {
    this.user = user;
  };

  clear = (): void => {
    this.user = null;
  };

  load = async (): Promise<SessionUser | null> => {
    try {
      const user = await me();
      this.user = user;
      return user;
    } catch (e: unknown) {
      if (e instanceof ApiError && e.status === 401) {
        this.user = null;
        return null;
      }
      throw e;
    }
  };
}

export const session = new SessionStore();
