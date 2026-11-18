import { signOut } from '#lib/auth/account.ts';
import { jobs } from '#lib/stores/jobs.svelte.ts';

class SignOutFlow {
  confirming = $state(false);
  signingOut = $state(false);

  request = (): void => {
    if (this.signingOut) return;
    if (jobs.activeTransfers === 0) void this.confirm();
    else this.confirming = true;
  };

  confirm = async (): Promise<void> => {
    if (this.signingOut) return;
    this.confirming = false;
    this.signingOut = true;
    try {
      await signOut();
    } finally {
      this.signingOut = false;
    }
  };

  cancel = (): void => {
    this.confirming = false;
  };
}

export const signOutFlow = new SignOutFlow();
