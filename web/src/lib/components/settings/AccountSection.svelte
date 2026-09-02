<script lang="ts">
  import { session } from '$lib/stores/session.svelte';
  import { logout } from '$lib/api/auth';
  import { mdiAccountCircleOutline, mdiLogoutVariant } from '@mdi/js';
  import { Badge, Icon, IconButton } from '@immich/ui';

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
  <section class="py-2">
    <div class="flex min-h-16 items-center gap-3">
      <Icon
        icon={mdiAccountCircleOutline}
        size="40px"
        class="shrink-0 text-dark/45"
        aria-hidden="true"
      />
      <div class="min-w-0 flex-1 text-xs">
        <div class="truncate text-sm font-medium">{session.user.name || session.user.email}</div>
        <div class="mt-0.5 flex min-w-0 flex-wrap items-center gap-1.5 text-dark/65">
          <span class="truncate font-mono">{session.user.email}</span>
          {#if session.user.is_admin}<Badge size="tiny" color="primary">admin</Badge>{/if}
          <Badge size="tiny" color="secondary">
            {session.user.auth_kind === 'password' ? 'password' : 'API key'}
          </Badge>
        </div>
      </div>
      <IconButton
        size="small"
        variant="ghost"
        color="secondary"
        icon={mdiLogoutVariant}
        title="Sign out"
        aria-label="Sign out"
        loading={signingOut}
        disabled={signingOut}
        onclick={() => void signOut()}
      />
    </div>
  </section>
{/if}
