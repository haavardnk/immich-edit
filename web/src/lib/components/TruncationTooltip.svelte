<script lang="ts">
  import { isTruncated } from '#lib/utils/truncation.ts';
  import { Tooltip } from '@immich/ui';
  import { mergeProps } from 'bits-ui';
  import type { Snippet } from 'svelte';

  let {
    text,
    child: trigger
  }: { text: string; child: Snippet<[{ props: Record<string, unknown> }]> } = $props();

  let truncated = $state(false);

  function measure(event: Event): void {
    if (event.currentTarget instanceof Element) truncated = isTruncated(event.currentTarget);
  }

  const measuring = { onpointerenter: measure, onfocus: measure };
</script>

<Tooltip {text} disabled={truncated ? undefined : true}>
  {#snippet child({ props })}
    {@render trigger({ props: mergeProps(measuring, props) })}
  {/snippet}
</Tooltip>
