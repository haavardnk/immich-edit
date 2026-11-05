<script lang="ts">
  import ElementTooltip from '#lib/components/ElementTooltip.svelte';
  import { isTruncated } from '#lib/utils/truncation.ts';
  import { mergeProps } from 'bits-ui';
  import type { Snippet } from 'svelte';

  let {
    text,
    force = false,
    child: trigger
  }: {
    text: string;
    force?: boolean;
    child: Snippet<[{ props: Record<string, unknown> }]>;
  } = $props();

  let truncated = $state(false);

  function measure(event: Event): void {
    if (event.currentTarget instanceof Element) truncated = isTruncated(event.currentTarget);
  }

  const measuring = { onpointerenter: measure, onfocus: measure };
</script>

<ElementTooltip {text} disabled={truncated || force ? undefined : true}>
  {#snippet child({ props })}
    {@render trigger({ props: mergeProps(measuring, props) })}
  {/snippet}
</ElementTooltip>
