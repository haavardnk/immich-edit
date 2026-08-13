<script lang="ts">
  import { session } from '$lib/stores/session.svelte';
  import { logout } from '$lib/api/auth';

  let signingOut = $state(false);

  async function signOut(): Promise<void> {
    if (signingOut) return;
    signingOut = true;
    try {
      await logout();
    } catch {
      /* ignore */
    }
    session.clear();
    window.location.replace('/login');
  }
</script>

{#if session.user}
  <section class="space-y-2">
    <h2 class="text-xs uppercase tracking-wider text-immich-dark-fg/50">Account</h2>
    <div class="flex items-center justify-between rounded bg-white/5 px-3 py-2">
      <div class="text-xs">
        <div class="font-medium">{session.user.name || session.user.email}</div>
        <div class="text-immich-dark-fg/50 font-mono">
          {session.user.email}
          {#if session.user.is_admin}<span class="text-immich-primary"> · admin</span>{/if}
          <span class="text-immich-dark-fg/40">
            · {session.user.auth_kind === 'password' ? 'password' : 'API key'}</span
          >
        </div>
      </div>
      <button
        class="px-3 py-1.5 rounded bg-white/5 hover:bg-white/10 text-xs disabled:opacity-50"
        onclick={() => void signOut()}
        disabled={signingOut}
      >
        {signingOut ? 'Signing out…' : 'Sign out'}
      </button>
    </div>
  </section>
{/if}
