<script lang="ts">
  import AccountBadges from '#lib/components/settings/AccountBadges.svelte';
  import ImmichServerLine from '#lib/components/shell/account/ImmichServerLine.svelte';
  import { session } from '#lib/stores/session.svelte.ts';
  import { signOutFlow } from '#lib/auth/signOutFlow.svelte.ts';
  import { mdiAccountCircleOutline, mdiLogoutVariant } from '@mdi/js';
  import { Icon, IconButton } from '@immich/ui';
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
          <AccountBadges user={session.user} />
        </div>
        <ImmichServerLine class="mt-0.5 truncate text-dark/65" />
      </div>
      <IconButton
        size="small"
        variant="ghost"
        color="secondary"
        icon={mdiLogoutVariant}
        title="Sign out"
        aria-label="Sign out"
        loading={signOutFlow.signingOut}
        disabled={signOutFlow.signingOut}
        onclick={signOutFlow.request}
      />
    </div>
  </section>
{/if}
