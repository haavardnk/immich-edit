<script lang="ts">
  import RebindDialog from '#lib/components/settings/RebindDialog.svelte';
  import ReconnectCard from '#lib/components/shell/ReconnectCard.svelte';
  import { Reconnect } from '#lib/connection/reconnect.svelte.ts';
  import { Button } from '@immich/ui';
  import { onMount } from 'svelte';

  let {
    probe,
    retry,
    admin = false
  }: { probe: () => Promise<boolean>; retry: () => void; admin?: boolean } = $props();

  let rebinding = $state(false);

  const reconnect = new Reconnect(
    () => probe(),
    () => retry()
  );

  onMount(() => reconnect.start());
</script>

<ReconnectCard
  title="Can't reach Immich."
  cause="immich-edit is running, but Immich doesn't answer."
  {reconnect}
>
  {#snippet actions()}
    {#if admin}
      <Button
        type="button"
        size="small"
        variant="ghost"
        color="secondary"
        class="bg-light-200/60"
        onclick={() => (rebinding = true)}
      >
        Change server…
      </Button>
    {/if}
  {/snippet}
</ReconnectCard>

{#if rebinding}
  <RebindDialog onClose={() => (rebinding = false)} />
{/if}
