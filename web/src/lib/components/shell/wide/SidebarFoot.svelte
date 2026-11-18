<script lang="ts">
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import SettingsButton from '#lib/components/shell/SettingsButton.svelte';
  import AvatarButton from '#lib/components/shell/account/AvatarButton.svelte';
  import SidebarJobs from '#lib/components/shell/wide/SidebarJobs.svelte';
  import { immichCapabilities } from '#lib/stores/immichCapabilities.svelte.ts';
  import { session } from '#lib/stores/session.svelte.ts';

  const name = $derived(session.user?.name || session.user?.email || '');
  const host = $derived(immichCapabilities.externalHost);
</script>

<div class="flex shrink-0 flex-col gap-2 px-2.5 pt-2 pb-2">
  <SidebarJobs />
  <div class="flex h-14 items-center gap-2 border-t border-hairline">
    <AvatarButton />
    <div class="min-w-0 flex-1 leading-tight">
      <TruncationTooltip text={name}>
        {#snippet child({ props })}
          <div {...props} data-truncate class="truncate text-sm text-light-950">{name}</div>
        {/snippet}
      </TruncationTooltip>
      {#if host}
        <TruncationTooltip text={host}>
          {#snippet child({ props })}
            <div {...props} data-truncate class="truncate text-xs text-dark/65">{host}</div>
          {/snippet}
        </TruncationTooltip>
      {/if}
    </div>
    <SettingsButton shape="round" class="size-11" />
  </div>
</div>
