<script lang="ts">
  import ExportPresetsGroup from '#lib/components/settings/ExportPresetsGroup.svelte';
  import SessionsGroup from '#lib/components/settings/SessionsGroup.svelte';
  import AccountCard from '#lib/components/shell/account/AccountCard.svelte';
  import { signOutFlow } from '#lib/auth/signOutFlow.svelte.ts';
  import { session } from '#lib/stores/session.svelte.ts';
  import { Button } from '@immich/ui';
  import { mdiLogoutVariant } from '@mdi/js';
</script>

<div class="flex flex-col gap-7 touch:gap-5">
  {#if session.user}
    <div
      data-setting-anchor="identity"
      class="flex items-center gap-4 rounded-xl bg-light-100 p-4 ring-1 ring-hairline touch:flex-col touch:items-stretch touch:rounded-2xl touch:ring-0"
    >
      <div class="min-w-0 flex-1">
        <AccountCard user={session.user} inset={false} />
      </div>
      <Button
        size="small"
        variant="ghost"
        color="secondary"
        class="shrink-0 bg-light-200/60 touch:min-h-11"
        leadingIcon={mdiLogoutVariant}
        loading={signOutFlow.signingOut}
        disabled={signOutFlow.signingOut}
        onclick={signOutFlow.request}
      >
        Sign out
      </Button>
    </div>
  {/if}
  <SessionsGroup />
  <ExportPresetsGroup />
</div>
