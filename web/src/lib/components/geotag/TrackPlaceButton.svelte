<script lang="ts">
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Button } from '@immich/ui';

  let { size = 'small', class: className = '' }: { size?: 'small' | 'large'; class?: string } =
    $props();

  const session = trackSession;

  const placed = $derived(session.counts.placed);
  const saving = $derived(session.timeWrites);
  const label = $derived(
    saving === 0
      ? `Place ${countOf(placed, 'photo')}`
      : placed === 0
        ? `Save time for ${countOf(saving, 'photo')}`
        : `Place ${placed} & save time`
  );
</script>

<Button
  {size}
  class={className}
  loading={session.placing}
  disabled={(placed === 0 && saving === 0) || session.placing}
  onclick={() => void session.place()}
>
  {label}
</Button>
