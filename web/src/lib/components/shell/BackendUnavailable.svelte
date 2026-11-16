<script lang="ts">
  import { live } from '#lib/api/health.ts';
  import ReconnectCard from '#lib/components/shell/ReconnectCard.svelte';
  import { Reconnect } from '#lib/connection/reconnect.svelte.ts';
  import { onMount } from 'svelte';

  let { retry }: { retry: () => void } = $props();

  const reconnect = new Reconnect(
    async () => {
      await live();
      return true;
    },
    () => retry()
  );

  onMount(() => reconnect.start());
</script>

{#if reconnect.offline}
  <ReconnectCard title="You're offline." {reconnect} />
{:else}
  <ReconnectCard
    title="Can't reach the immich-edit server."
    cause="It may still be starting up."
    {reconnect}
  />
{/if}
